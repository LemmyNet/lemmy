use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, check_community_ban, get_local_user_view_from_jwt, post::*};
use lemmy_apub::activities::{post::create_or_update::CreateOrUpdatePost, CreateOrUpdateType};
use lemmy_db_queries::{source::post::Post_, Crud};
use lemmy_db_schema::{naive_now, source::post::*};
use lemmy_utils::{
  request::fetch_site_data,
  utils::{check_slurs_opt, clean_url_params, is_valid_post_title},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &EditPost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    check_slurs_opt(&data.name)?;
    check_slurs_opt(&data.body)?;

    if let Some(name) = &data.name {
      if !is_valid_post_title(name) {
        return Err(ApiError::err("invalid_post_title").into());
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

    // Verify that only the creator can edit
    if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
      return Err(ApiError::err("no_post_edit_allowed").into());
    }

    // Fetch post links and Pictrs cached image
    let data_url = data.url.as_ref();
    let (metadata_res, pictrs_thumbnail) = fetch_site_data(context.client(), data_url).await;
    let (embed_title, embed_description, embed_html) = metadata_res
      .map(|u| (u.title, u.description, u.html))
      .unwrap_or((None, None, None));

    let post_form = PostForm {
      creator_id: orig_post.creator_id.to_owned(),
      community_id: orig_post.community_id,
      name: data.name.to_owned().unwrap_or(orig_post.name),
      url: data_url.map(|u| clean_url_params(u.to_owned()).into()),
      body: data.body.to_owned(),
      nsfw: data.nsfw,
      updated: Some(naive_now()),
      embed_title,
      embed_description,
      embed_html,
      thumbnail_url: pictrs_thumbnail.map(|u| u.into()),
      ..PostForm::default()
    };

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

        return Err(ApiError::err(err_type).into());
      }
    };

    // Send apub update
    CreateOrUpdatePost::send(
      &updated_post,
      &local_user_view.person,
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
