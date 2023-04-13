use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{EditPost, PostResponse},
  request::fetch_site_data,
  utils::{check_community_ban, get_local_user_view_from_jwt, local_site_to_slur_regex},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, naive_now},
};
use lemmy_utils::{
  error::LemmyError,
  utils::{
    slurs::check_slurs_opt,
    validation::{clean_url_params, is_valid_body_field, is_valid_post_title},
  },
  ConnectionId,
};

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
    let local_site = LocalSite::read(context.pool()).await?;

    let data_url = data.url.as_ref();

    // TODO No good way to handle a clear.
    // Issue link: https://github.com/LemmyNet/lemmy/issues/2287
    let url = Some(data_url.map(clean_url_params).map(Into::into));
    let body = diesel_option_overwrite(&data.body);

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs_opt(&data.name, &slur_regex)?;
    check_slurs_opt(&data.body, &slur_regex)?;

    if let Some(name) = &data.name {
      is_valid_post_title(name)?;
    }

    is_valid_body_field(&data.body)?;

    let post_id = data.post_id;
    let orig_post = Post::read(context.pool(), post_id).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;

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
    CommunityLanguage::is_allowed_community_language(
      context.pool(),
      language_id,
      orig_post.community_id,
    )
    .await?;

    let post_form = PostUpdateForm::builder()
      .name(data.name.clone())
      .url(url)
      .body(body)
      .nsfw(data.nsfw)
      .embed_title(embed_title)
      .embed_description(embed_description)
      .embed_video_url(embed_video_url)
      .language_id(data.language_id)
      .thumbnail_url(Some(thumbnail_url))
      .updated(Some(Some(naive_now())))
      .build();

    let post_id = data.post_id;
    let res = Post::update(context.pool(), post_id, &post_form).await;
    if let Err(e) = res {
      let err_type = if e.to_string() == "value too long for type character varying(200)" {
        "post_title_too_long"
      } else {
        "couldnt_update_post"
      };

      return Err(LemmyError::from_error_message(e, err_type));
    }

    context
      .send_post_ws_message(
        &UserOperationCrud::EditPost,
        data.post_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await
  }
}
