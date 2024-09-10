use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::SaveUserSettings,
  request::replace_image,
  utils::{
    get_url_blocklist,
    local_site_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_api,
    send_verification_email,
  },
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    local_user::{LocalUser, LocalUserUpdateForm},
    local_user_vote_display_mode::{LocalUserVoteDisplayMode, LocalUserVoteDisplayModeUpdateForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::validation::{is_valid_bio_field, is_valid_display_name, is_valid_matrix_id},
};
use std::ops::Deref;

#[tracing::instrument(skip(context))]
pub async fn save_user_settings(
  data: Json<SaveUserSettings>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;

  let slur_regex = local_site_to_slur_regex(&site_view.local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let bio = diesel_string_update(
    process_markdown_opt(&data.bio, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  let avatar = diesel_url_update(data.avatar.as_deref())?;
  replace_image(&avatar, &local_user_view.person.avatar, &context).await?;
  let avatar = proxy_image_link_opt_api(avatar, &context).await?;

  let banner = diesel_url_update(data.banner.as_deref())?;
  replace_image(&banner, &local_user_view.person.banner, &context).await?;
  let banner = proxy_image_link_opt_api(banner, &context).await?;

  let display_name = diesel_string_update(data.display_name.as_deref());
  let matrix_user_id = diesel_string_update(data.matrix_user_id.as_deref());
  let email_deref = data.email.as_deref().map(str::to_lowercase);
  let email = diesel_string_update(email_deref.as_deref());

  if let Some(Some(email)) = &email {
    let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
    // if email was changed, check that it is not taken and send verification mail
    if previous_email.deref() != email {
      LocalUser::check_is_email_taken(&mut context.pool(), email).await?;
      send_verification_email(
        &local_user_view,
        email,
        &mut context.pool(),
        context.settings(),
      )
      .await?;
    }
  }

  // When the site requires email, make sure email is not Some(None). IE, an overwrite to a None
  // value
  if let Some(email) = &email {
    if email.is_none() && site_view.local_site.require_email_verification {
      Err(LemmyErrorType::EmailRequired)?
    }
  }

  if let Some(Some(bio)) = &bio {
    is_valid_bio_field(bio)?;
  }

  if let Some(Some(display_name)) = &display_name {
    is_valid_display_name(
      display_name.trim(),
      site_view.local_site.actor_name_max_length as usize,
    )?;
  }

  if let Some(Some(matrix_user_id)) = &matrix_user_id {
    is_valid_matrix_id(matrix_user_id)?;
  }

  let local_user_id = local_user_view.local_user.id;
  let person_id = local_user_view.person.id;
  let default_listing_type = data.default_listing_type;
  let default_sort_type = data.default_sort_type;

  let person_form = PersonUpdateForm {
    display_name,
    bio,
    matrix_user_id,
    bot_account: data.bot_account,
    avatar,
    banner,
    ..Default::default()
  };

  // Ignore errors, because 'no fields updated' will return an error.
  // https://github.com/LemmyNet/lemmy/issues/4076
  Person::update(&mut context.pool(), person_id, &person_form)
    .await
    .ok();

  if let Some(discussion_languages) = data.discussion_languages.clone() {
    LocalUserLanguage::update(&mut context.pool(), discussion_languages, local_user_id).await?;
  }

  let local_user_form = LocalUserUpdateForm {
    email,
    show_avatars: data.show_avatars,
    show_read_posts: data.show_read_posts,
    send_notifications_to_email: data.send_notifications_to_email,
    show_nsfw: data.show_nsfw,
    blur_nsfw: data.blur_nsfw,
    auto_expand: data.auto_expand,
    show_bot_accounts: data.show_bot_accounts,
    default_sort_type,
    default_listing_type,
    theme: data.theme.clone(),
    interface_language: data.interface_language.clone(),
    open_links_in_new_tab: data.open_links_in_new_tab,
    infinite_scroll_enabled: data.infinite_scroll_enabled,
    post_listing_mode: data.post_listing_mode,
    enable_keyboard_navigation: data.enable_keyboard_navigation,
    enable_animated_images: data.enable_animated_images,
    collapse_bot_comments: data.collapse_bot_comments,
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &local_user_form).await?;

  // Update the vote display modes
  let vote_display_modes_form = LocalUserVoteDisplayModeUpdateForm {
    score: data.show_scores,
    upvotes: data.show_upvotes,
    downvotes: data.show_downvotes,
    upvote_percentage: data.show_upvote_percentage,
  };
  LocalUserVoteDisplayMode::update(&mut context.pool(), local_user_id, &vote_display_modes_form)
    .await?;

  Ok(Json(SuccessResponse::default()))
}
