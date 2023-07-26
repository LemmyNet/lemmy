use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{EditPost, PostResponse},
  request::fetch_site_data,
  utils::{
    check_community_ban,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    sanitize_html_opt,
  },
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
    validation::{check_url_scheme, clean_url_params, is_valid_body_field, is_valid_post_title},
  },
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &EditPost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let data_url = data.url.as_ref();

    // TODO No good way to handle a clear.
    // Issue link: https://github.com/LemmyNet/lemmy/issues/2287
    let url = Some(data_url.map(clean_url_params).map(Into::into));

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs_opt(&data.name, &slur_regex)?;
    check_slurs_opt(&data.body, &slur_regex)?;

    if let Some(name) = &data.name {
      is_valid_post_title(name)?;
    }

    is_valid_body_field(&data.body, true)?;
    check_url_scheme(&data.url)?;

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
      fetch_site_data(context.client(), context.settings(), data_url, true).await;
    let (embed_title, embed_description, embed_video_url) = metadata_res
      .map(|u| (Some(u.title), Some(u.description), Some(u.embed_video_url)))
      .unwrap_or_default();

    let name = sanitize_html_opt(&data.name);
    let body = sanitize_html_opt(&data.body);
    let body = diesel_option_overwrite(body);
    let embed_title = embed_title.map(|e| sanitize_html_opt(&e));
    let embed_description = embed_description.map(|e| sanitize_html_opt(&e));

    let language_id = self.language_id;
    CommunityLanguage::is_allowed_community_language(
      context.pool(),
      language_id,
      orig_post.community_id,
    )
    .await?;

    let post_form = PostUpdateForm::builder()
      .name(name)
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
    Post::update(context.pool(), post_id, &post_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_post"))?;

    build_post_response(
      context,
      orig_post.community_id,
      local_user_view.person.id,
      post_id,
    )
    .await
  }
}
