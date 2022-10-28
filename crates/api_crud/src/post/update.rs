use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{EditPost, PostResponse},
  request::fetch_site_data,
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
    local_site_to_slur_regex,
  },
};
use lemmy_apub::protocol::activities::{
  create_or_update::post::CreateOrUpdatePost,
  CreateOrUpdateType,
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::diesel_option_overwrite,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs_opt, clean_url_params, is_valid_post_title},
  ConnectionId,
};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &EditPost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    let local_site = blocking(context.pool(), LocalSite::read).await??;

    let data_url = data.url.as_ref();

    // TODO No good way to handle a clear.
    // Issue link: https://github.com/LemmyNet/lemmy/issues/2287
    let url = Some(data_url.map(clean_url_params).map(Into::into));
    let body = diesel_option_overwrite(&data.body);

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs_opt(&data.name, &slur_regex)?;
    check_slurs_opt(&data.body, &slur_regex)?;

    if let Some(name) = &data.name {
      if !is_valid_post_title(name) {
        return Err(LemmyError::from_message("invalid_post_title"));
      }
    }

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can edit
    if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
      return Err(LemmyError::from_message("no_post_edit_allowed"));
    }

    // Fetch post links and Pictrs cached image
    let data_url = data.url.as_ref();
    let (metadata_res, thumbnail_url) =
      fetch_site_data(context.client(), context.settings(), data_url).await;
    let (embed_title, embed_description, embed_video_url) = metadata_res
      .map(|u| (Some(u.title), Some(u.description), Some(u.embed_video_url)))
      .unwrap_or_default();

    let language_id = self.language_id;
    blocking(context.pool(), move |conn| {
      CommunityLanguage::is_allowed_community_language(conn, language_id, orig_post.community_id)
    })
    .await??;

    let post_form = PostUpdateForm::builder()
      .name(data.name.to_owned())
      .url(url)
      .body(body)
      .nsfw(data.nsfw)
      .embed_title(embed_title)
      .embed_description(embed_description)
      .embed_video_url(embed_video_url)
      .language_id(data.language_id)
      .thumbnail_url(Some(thumbnail_url))
      .build();

    let post_id = data.post_id;
    let res = blocking(context.pool(), move |conn| {
      Post::update(conn, post_id, &post_form)
    })
    .await?;
    let updated_post: Post = match res {
      Ok(post) => post,
      Err(e) => {
        let err_type = if e.to_string() == "value too long for type character varying(200)" {
          "post_title_too_long"
        } else {
          "couldnt_update_post"
        };

        return Err(LemmyError::from_error_message(e, err_type));
      }
    };

    // Send apub update
    CreateOrUpdatePost::send(
      updated_post.into(),
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperationCrud::EditPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
