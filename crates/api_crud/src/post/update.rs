use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, check_community_ban, get_local_user_view_from_jwt, post::*};
use lemmy_apub::ApubObjectType;
use lemmy_db_queries::{source::post::Post_, Crud};
use lemmy_db_schema::{naive_now, source::post::*};
use lemmy_db_views::post_view::PostView;
use lemmy_utils::{
  request::fetch_iframely_and_pictrs_data,
  utils::{check_slurs, check_slurs_opt, is_valid_post_title},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{messages::SendPost, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &EditPost = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs_opt(&data.body)?;

    if !is_valid_post_title(&data.name) {
      return Err(ApiError::err("invalid_post_title").into());
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

    // Fetch Iframely and Pictrs cached image
    let data_url = data.url.as_ref();
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(context.client(), data_url).await;

    let post_form = PostForm {
      creator_id: orig_post.creator_id.to_owned(),
      community_id: orig_post.community_id,
      name: data.name.trim().to_owned(),
      url: data_url.map(|u| u.to_owned().into()),
      body: data.body.to_owned(),
      nsfw: data.nsfw,
      updated: Some(naive_now()),
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
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
    updated_post
      .send_update(&local_user_view.person, context)
      .await?;

    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(local_user_view.person.id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperationCrud::EditPost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
