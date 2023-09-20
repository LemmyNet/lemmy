use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, SaveUserSettings},
  utils::{local_user_view_from_jwt, sanitize_html_api_opt, send_verification_email},
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
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  claims::Claims,
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::validation::{is_valid_bio_field, is_valid_display_name, is_valid_matrix_id},
};

#[tracing::instrument(skip(context))]
pub async fn save_user_settings(
  data: Json<SaveUserSettings>,
  context: Data<LemmyContext>,
) -> Result<Json<LoginResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let bio = sanitize_html_api_opt(&data.bio);
  let display_name = sanitize_html_api_opt(&data.display_name);

  let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
  let banner = diesel_option_overwrite_to_url(&data.banner)?;
  let bio = diesel_option_overwrite(bio);
  let display_name = diesel_option_overwrite(display_name);
  let matrix_user_id = diesel_option_overwrite(data.matrix_user_id.clone());
  let email_deref = data.email.as_deref().map(str::to_lowercase);
  let email = diesel_option_overwrite(email_deref.clone());

  if let Some(Some(email)) = &email {
    let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
    // Only send the verification email if there was an email change
    if previous_email.ne(email) {
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
  let theme = sanitize_html_api_opt(&data.theme);

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
    show_new_post_notifs: data.show_new_post_notifs,
    send_notifications_to_email: data.send_notifications_to_email,
    show_nsfw: data.show_nsfw,
    blur_nsfw: data.blur_nsfw,
    auto_expand: data.auto_expand,
    show_bot_accounts: data.show_bot_accounts,
    show_scores: data.show_scores,
    default_sort_type,
    default_listing_type,
    theme,
    interface_language: data.interface_language.clone(),
    open_links_in_new_tab: data.open_links_in_new_tab,
    infinite_scroll_enabled: data.infinite_scroll_enabled,
    ..Default::default()
  };

  let local_user_res =
    LocalUser::update(&mut context.pool(), local_user_id, &local_user_form).await;
  let updated_local_user = match local_user_res {
    Ok(u) => u,
    Err(e) => {
      let err_type = if e.to_string()
        == "duplicate key value violates unique constraint \"local_user_email_key\""
      {
        LemmyErrorType::EmailAlreadyExists
      } else {
        LemmyErrorType::UserAlreadyExists
      };

      return Err(e).with_lemmy_type(err_type);
    }
  };

  // Return the jwt
  Ok(Json(LoginResponse {
    jwt: Some(
      Claims::jwt(
        updated_local_user.id.0,
        &context.secret().jwt_secret,
        &context.settings().hostname,
      )?
      .into(),
    ),
    verify_email_sent: false,
    registration_created: false,
  }))
}
