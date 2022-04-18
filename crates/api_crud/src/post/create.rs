use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  check_community_deleted_or_removed,
  get_local_user_view_from_jwt,
  honeypot_check,
  mark_post_as_read,
  post::*,
};
use lemmy_apub::{
  generate_local_apub_endpoint,
  objects::post::ApubPost,
  protocol::activities::{create_or_update::post::CreateOrUpdatePost, CreateOrUpdateType},
  EndpointType,
};
use lemmy_db_schema::{
  source::post::{Post, PostForm, PostLike, PostLikeForm},
  traits::{Crud, Likeable},
};
use lemmy_utils::{
  request::fetch_site_data,
  utils::{
    check_slurs,
    check_slurs_opt,
    clean_optional_text,
    clean_url_params,
    is_valid_post_title,
  },
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};
use tracing::{warn, Instrument};
use url::Url;
use webmention::{Webmention, WebmentionError};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreatePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let slur_regex = &context.settings().slur_regex();
    check_slurs(&data.name, slur_regex)?;
    check_slurs_opt(&data.body, slur_regex)?;
    honeypot_check(&data.honeypot)?;

    if !is_valid_post_title(&data.name) {
      return Err(LemmyError::from_message("invalid_post_title"));
    }

    check_community_ban(local_user_view.person.id, data.community_id, context.pool()).await?;
    check_community_deleted_or_removed(data.community_id, context.pool()).await?;

    // Fetch post links and pictrs cached image
    let data_url = data.url.as_ref();
    let (metadata_res, pictrs_thumbnail) =
      fetch_site_data(context.client(), &context.settings(), data_url).await;
    let (embed_title, embed_description, embed_html) = metadata_res
      .map(|u| (u.title, u.description, u.html))
      .unwrap_or((None, None, None));

    let post_form = PostForm {
      name: data.name.trim().to_owned(),
      url: data_url.map(|u| clean_url_params(u.to_owned()).into()),
      body: clean_optional_text(&data.body),
      community_id: data.community_id,
      creator_id: local_user_view.person.id,
      nsfw: data.nsfw,
      embed_title,
      embed_description,
      embed_html,
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

          return Err(LemmyError::from_error_message(e, err_type));
        }
      };

    let inserted_post_id = inserted_post.id;
    let protocol_and_hostname = context.settings().get_protocol_and_hostname();
    let updated_post = blocking(context.pool(), move |conn| -> Result<Post, LemmyError> {
      let apub_id = generate_local_apub_endpoint(
        EndpointType::Post,
        &inserted_post_id.to_string(),
        &protocol_and_hostname,
      )?;
      Ok(Post::update_ap_id(conn, inserted_post_id, apub_id)?)
    })
    .await?
    .map_err(|e| e.with_message("couldnt_create_post"))?;

    // They like their own post by default
    let person_id = local_user_view.person.id;
    let post_id = inserted_post.id;
    let like_form = PostLikeForm {
      post_id,
      person_id,
      score: 1,
    };

    let like = move |conn: &'_ _| PostLike::like(conn, &like_form);
    blocking(context.pool(), like)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_post"))?;

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    if let Some(url) = &updated_post.url {
      let mut webmention =
        Webmention::new::<Url>(updated_post.ap_id.clone().into(), url.clone().into())?;
      webmention.set_checked(true);
      match webmention
        .send()
        .instrument(tracing::info_span!("Sending webmention"))
        .await
      {
        Ok(_) => {}
        Err(WebmentionError::NoEndpointDiscovered(_)) => {}
        Err(e) => warn!("Failed to send webmention: {}", e),
      }
    }

    let apub_post: ApubPost = updated_post.into();
    CreateOrUpdatePost::send(
      apub_post.clone(),
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Create,
      context,
    )
    .await?;

    send_post_ws_message(
      inserted_post.id,
      UserOperationCrud::CreatePost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
