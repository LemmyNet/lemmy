use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::SaveUserSettings,
  utils::send_verification_email,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::validation::{is_valid_bio_field, is_valid_display_name, is_valid_matrix_id},
};

#[tracing::instrument(skip(context))]
pub async fn save_user_settings(
  data: Json<SaveUserSettings>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
  let banner = diesel_option_overwrite_to_url(&data.banner)?;
  let bio = diesel_option_overwrite(data.bio.clone());
  let display_name = diesel_option_overwrite(data.display_name.clone());
  let matrix_user_id = diesel_option_overwrite(data.matrix_user_id.clone());
  let email_deref = data.email.as_deref().map(str::to_lowercase);
  let email = diesel_option_overwrite(email_deref.clone());

  if let Some(Some(email)) = &email {
    let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
    // if email was changed, check that it is not taken and send verification mail
    if &previous_email != email {
      if LocalUser::is_email_taken(&mut context.pool(), email).await? {
        return Err(LemmyErrorType::EmailAlreadyExists)?;
      }
      send_verification_email(
        &local_user_view,
        email,
        &mut context.pool(),
        context.settings(),
      )
      .await?;
    }
  }

  // When the site requires email, make sure email is not Some(None). IE, an overwrite to a None value
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

  Person::update(&mut context.pool(), person_id, &person_form)
    .await
    .with_lemmy_type(LemmyErrorType::UserAlreadyExists)?;

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
    show_scores: data.show_scores,
    default_sort_type,
    default_listing_type,
    theme: data.theme.clone(),
    interface_language: data.interface_language.clone(),
    open_links_in_new_tab: data.open_links_in_new_tab,
    infinite_scroll_enabled: data.infinite_scroll_enabled,
    enable_keyboard_navigation: data.enable_keyboard_navigation,
    enable_animated_avatars: data.enable_animated_avatars,
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &local_user_form).await?;

  Ok(Json(SuccessResponse::default()))
}
