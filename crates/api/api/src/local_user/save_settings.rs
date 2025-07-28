use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    keyword_block::LocalUserKeywordBlock,
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
  utils::{diesel_opt_number_update, diesel_string_update},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{SaveUserSettings, SuccessResponse},
  SiteView,
};
use lemmy_email::account::send_verification_email;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::validation::{
    check_blocking_keywords_are_valid,
    is_valid_bio_field,
    is_valid_display_name,
    is_valid_matrix_id,
  },
};
use std::ops::Deref;

pub async fn save_user_settings(
  data: Json<SaveUserSettings>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let bio = diesel_string_update(
    process_markdown_opt(&data.bio, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  let display_name = diesel_string_update(data.display_name.as_deref());
  let matrix_user_id = diesel_string_update(data.matrix_user_id.as_deref());
  let email_deref = data.email.as_deref().map(str::to_lowercase);
  let email = diesel_string_update(email_deref.as_deref());

  if let Some(Some(email)) = email.clone() {
    let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
    // if email was changed, check that it is not taken and send verification mail
    if previous_email.deref() != email {
      LocalUser::check_is_email_taken(&mut context.pool(), &email).await?;
      send_verification_email(
        &site_view.local_site,
        &local_user_view,
        email.into(),
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
      site_view.local_site.actor_name_max_length,
    )?;
  }

  if let Some(Some(matrix_user_id)) = &matrix_user_id {
    is_valid_matrix_id(matrix_user_id)?;
  }

  if let Some(send_notifications_to_email) = data.send_notifications_to_email {
    if site_view.local_site.disable_email_notifications && send_notifications_to_email {
      return Err(LemmyErrorType::EmailNotificationsDisabled.into());
    }
  }

  let local_user_id = local_user_view.local_user.id;
  let person_id = local_user_view.person.id;
  let default_listing_type = data.default_listing_type;
  let default_post_sort_type = data.default_post_sort_type;
  let default_post_time_range_seconds =
    diesel_opt_number_update(data.default_post_time_range_seconds);
  let default_items_per_page = data.default_items_per_page;
  let default_comment_sort_type = data.default_comment_sort_type;

  let person_form = PersonUpdateForm {
    display_name,
    bio,
    matrix_user_id,
    bot_account: data.bot_account,
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

  if let Some(blocking_keywords) = data.blocking_keywords.clone() {
    let trimmed_blocking_keywords = blocking_keywords
      .iter()
      .map(|blocking_keyword| blocking_keyword.trim().to_string())
      .collect();
    check_blocking_keywords_are_valid(&trimmed_blocking_keywords)?;
    LocalUserKeywordBlock::update(
      &mut context.pool(),
      trimmed_blocking_keywords,
      local_user_id,
    )
    .await?;
  }

  let local_user_form = LocalUserUpdateForm {
    email,
    show_avatars: data.show_avatars,
    show_read_posts: data.show_read_posts,
    send_notifications_to_email: data.send_notifications_to_email,
    show_nsfw: data.show_nsfw,
    blur_nsfw: data.blur_nsfw,
    show_bot_accounts: data.show_bot_accounts,
    default_post_sort_type,
    default_post_time_range_seconds,
    default_comment_sort_type,
    default_listing_type,
    default_items_per_page,
    theme: data.theme.clone(),
    interface_language: data.interface_language.clone(),
    open_links_in_new_tab: data.open_links_in_new_tab,
    infinite_scroll_enabled: data.infinite_scroll_enabled,
    post_listing_mode: data.post_listing_mode,
    enable_keyboard_navigation: data.enable_keyboard_navigation,
    enable_animated_images: data.enable_animated_images,
    enable_private_messages: data.enable_private_messages,
    collapse_bot_comments: data.collapse_bot_comments,
    auto_mark_fetched_posts_as_read: data.auto_mark_fetched_posts_as_read,
    hide_media: data.hide_media,
    // Update the vote display modes
    show_score: data.show_scores,
    show_upvotes: data.show_upvotes,
    show_downvotes: data.show_downvotes,
    show_upvote_percentage: data.show_upvote_percentage,
    show_person_votes: data.show_person_votes,
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &local_user_form).await?;

  Ok(Json(SuccessResponse::default()))
}
