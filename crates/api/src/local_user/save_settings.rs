use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{LoginResponse, SaveUserSettings},
  utils::{blocking, get_local_user_view_from_jwt, send_verification_email},
};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    local_site::LocalSite,
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url},
};
use lemmy_utils::{
  claims::Claims,
  error::LemmyError,
  utils::{is_valid_display_name, is_valid_matrix_id},
  ConnectionId,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for SaveUserSettings {
  type Response = LoginResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &SaveUserSettings = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    let local_site = blocking(context.pool(), LocalSite::read).await??;

    let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let bio = diesel_option_overwrite(&data.bio);
    let display_name = diesel_option_overwrite(&data.display_name);
    let matrix_user_id = diesel_option_overwrite(&data.matrix_user_id);
    let bot_account = data.bot_account;
    let email_deref = data.email.as_deref().map(|e| e.to_lowercase());
    let email = diesel_option_overwrite(&email_deref);

    if let Some(Some(email)) = &email {
      let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
      // Only send the verification email if there was an email change
      if previous_email.ne(email) {
        send_verification_email(&local_user_view, email, context.pool(), context.settings())
          .await?;
      }
    }

    // When the site requires email, make sure email is not Some(None). IE, an overwrite to a None value
    if let Some(email) = &email {
      if email.is_none() && local_site.require_email_verification {
        return Err(LemmyError::from_message("email_required"));
      }
    }

    if let Some(Some(bio)) = &bio {
      if bio.chars().count() > 300 {
        return Err(LemmyError::from_message("bio_length_overflow"));
      }
    }

    if let Some(Some(display_name)) = &display_name {
      if !is_valid_display_name(
        display_name.trim(),
        local_site.actor_name_max_length as usize,
      ) {
        return Err(LemmyError::from_message("invalid_username"));
      }
    }

    if let Some(Some(matrix_user_id)) = &matrix_user_id {
      if !is_valid_matrix_id(matrix_user_id) {
        return Err(LemmyError::from_message("invalid_matrix_id"));
      }
    }

    let local_user_id = local_user_view.local_user.id;
    let person_id = local_user_view.person.id;
    let default_listing_type = data.default_listing_type;
    let default_sort_type = data.default_sort_type;

    let person_form = PersonUpdateForm::builder()
      .display_name(display_name)
      .bio(bio)
      .matrix_user_id(matrix_user_id)
      .bot_account(bot_account)
      .avatar(avatar)
      .banner(banner)
      .build();

    blocking(context.pool(), move |conn| {
      Person::update(conn, person_id, &person_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "user_already_exists"))?;

    if let Some(discussion_languages) = data.discussion_languages.clone() {
      blocking(context.pool(), move |conn| {
        LocalUserLanguage::update(conn, discussion_languages, local_user_id)
      })
      .await??;
    }

    let local_user_form = LocalUserUpdateForm::builder()
      .email(email)
      .show_avatars(data.show_avatars)
      .show_read_posts(data.show_read_posts)
      .show_new_post_notifs(data.show_new_post_notifs)
      .send_notifications_to_email(data.send_notifications_to_email)
      .show_nsfw(data.show_nsfw)
      .show_bot_accounts(data.show_bot_accounts)
      .show_scores(data.show_scores)
      .default_sort_type(default_sort_type)
      .default_listing_type(default_listing_type)
      .theme(data.theme.to_owned())
      .interface_language(data.interface_language.to_owned())
      .build();

    let local_user_res = blocking(context.pool(), move |conn| {
      LocalUser::update(conn, local_user_id, &local_user_form)
    })
    .await?;
    let updated_local_user = match local_user_res {
      Ok(u) => u,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"local_user_email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        return Err(LemmyError::from_error_message(e, err_type));
      }
    };

    // Return the jwt
    Ok(LoginResponse {
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
    })
  }
}
