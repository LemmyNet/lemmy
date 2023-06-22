use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePost, PostResponse},
  request::fetch_site_data,
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    generate_local_apub_endpoint,
    honeypot_check,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    mark_post_as_read,
    EndpointType,
  },
};
use lemmy_db_schema::{
  impls::actor_language::default_post_language,
  source::{
    actor_language::CommunityLanguage,
    community::Community,
    local_site::LocalSite,
    post::{Post, PostInsertForm, PostLike, PostLikeForm, PostUpdateForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{
  error::LemmyError,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::{clean_url_params, is_valid_body_field, is_valid_post_title},
  },
};
use tracing::{warn, Instrument};
use url::Url;
use webmention::{Webmention, WebmentionError};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreatePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &CreatePost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs(&data.name, &slur_regex)?;
    check_slurs_opt(&data.body, &slur_regex)?;
    honeypot_check(&data.honeypot)?;

    let data_url = data.url.as_ref();
    let url = data_url.map(clean_url_params).map(Into::into); // TODO no good way to handle a "clear"

    is_valid_post_title(&data.name)?;
    is_valid_body_field(&data.body, true)?;

    check_community_ban(local_user_view.person.id, data.community_id, context.pool()).await?;
    check_community_deleted_or_removed(data.community_id, context.pool()).await?;

    let community_id = data.community_id;
    let community = Community::read(context.pool(), community_id).await?;
    if community.posting_restricted_to_mods {
      let community_id = data.community_id;
      let is_mod = CommunityView::is_mod_or_admin(
        context.pool(),
        local_user_view.local_user.person_id,
        community_id,
      )
      .await?;
      if !is_mod {
        return Err(LemmyError::from_message("only_mods_can_post_in_community"));
      }
    }

    // Fetch post links and pictrs cached image
    let (metadata_res, thumbnail_url) =
      fetch_site_data(context.client(), context.settings(), data_url).await;
    let (embed_title, embed_description, embed_video_url) = metadata_res
      .map(|u| (u.title, u.description, u.embed_video_url))
      .unwrap_or_default();

    let language_id = match data.language_id {
      Some(lid) => Some(lid),
      None => {
        default_post_language(context.pool(), community_id, local_user_view.local_user.id).await?
      }
    };
    CommunityLanguage::is_allowed_community_language(context.pool(), language_id, community_id)
      .await?;

    let post_form = PostInsertForm::builder()
      .name(data.name.trim().to_owned())
      .url(url)
      .body(data.body.clone())
      .community_id(data.community_id)
      .creator_id(local_user_view.person.id)
      .nsfw(data.nsfw)
      .embed_title(embed_title)
      .embed_description(embed_description)
      .embed_video_url(embed_video_url)
      .language_id(language_id)
      .thumbnail_url(thumbnail_url)
      .build();

    let inserted_post = match Post::create(context.pool(), &post_form).await {
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
    let apub_id = generate_local_apub_endpoint(
      EndpointType::Post,
      &inserted_post_id.to_string(),
      &protocol_and_hostname,
    )?;
    let updated_post = Post::update(
      context.pool(),
      inserted_post_id,
      &PostUpdateForm::builder().ap_id(Some(apub_id)).build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_post"))?;

    // They like their own post by default
    let person_id = local_user_view.person.id;
    let post_id = inserted_post.id;
    let like_form = PostLikeForm {
      post_id,
      person_id,
      score: 1,
    };

    PostLike::like(context.pool(), &like_form)
      .await
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

    build_post_response(context, community_id, person_id, post_id).await
  }
}
