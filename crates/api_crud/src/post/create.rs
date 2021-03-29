use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, check_community_ban, get_local_user_view_from_jwt, post::*};
use lemmy_apub::{generate_apub_endpoint, ApubLikeableType, ApubObjectType, EndpointType};
use lemmy_db_queries::{source::post::Post_, Crud, Likeable};
use lemmy_db_schema::source::post::*;
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
impl PerformCrud for CreatePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePost = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs_opt(&data.body)?;

    if !is_valid_post_title(&data.name) {
      return Err(ApiError::err("invalid_post_title").into());
    }

    check_community_ban(local_user_view.person.id, data.community_id, context.pool()).await?;

    // Fetch Iframely and pictrs cached image
    let data_url = data.url.as_ref();
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(context.client(), data_url).await;

    let post_form = PostForm {
      name: data.name.trim().to_owned(),
      url: data_url.map(|u| u.to_owned().into()),
      body: data.body.to_owned(),
      community_id: data.community_id,
      creator_id: local_user_view.person.id,
      nsfw: data.nsfw,
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictrs_thumbnail.map(|u| u.into()),
      ..PostForm::default()
    };

    let inserted_post =
      match blocking(context.pool(), move |conn| Post::create(conn, &post_form)).await? {
        Ok(post) => post,
        Err(e) => {
          let err_type = if e.to_string() == "value too long for type character varying(200)" {
            "post_title_too_long"
          } else {
            "couldnt_create_post"
          };

          return Err(ApiError::err(err_type).into());
        }
      };

    let inserted_post_id = inserted_post.id;
    let updated_post = match blocking(context.pool(), move |conn| -> Result<Post, LemmyError> {
      let apub_id = generate_apub_endpoint(EndpointType::Post, &inserted_post_id.to_string())?;
      Ok(Post::update_ap_id(conn, inserted_post_id, apub_id)?)
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_create_post").into()),
    };

    updated_post
      .send_create(&local_user_view.person, context)
      .await?;

    // They like their own post by default
    let like_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: local_user_view.person.id,
      score: 1,
    };

    let like = move |conn: &'_ _| PostLike::like(conn, &like_form);
    if blocking(context.pool(), like).await?.is_err() {
      return Err(ApiError::err("couldnt_like_post").into());
    }

    updated_post
      .send_like(&local_user_view.person, context)
      .await?;

    // Refetch the view
    let inserted_post_id = inserted_post.id;
    let post_view = match blocking(context.pool(), move |conn| {
      PostView::read(conn, inserted_post_id, Some(local_user_view.person.id))
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperationCrud::CreatePost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
