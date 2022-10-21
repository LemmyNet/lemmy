use crate::PerformCrud;
use activitypub_federation::core::signatures::generate_actor_keypair;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{LoginResponse, Register},
  utils::{
    blocking,
    honeypot_check,
    local_site_to_slur_regex,
    password_length_check,
    send_new_applicant_email_to_admins,
    send_verification_email,
  },
};
use lemmy_apub::{
  generate_inbox_url,
  generate_local_apub_endpoint,
  generate_shared_inbox_url,
  EndpointType,
};
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  source::{
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_db_views_actor::structs::PersonViewSafe;
use lemmy_utils::{
  claims::Claims,
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt, is_valid_actor_name},
  ConnectionId,
};
use lemmy_websocket::{messages::CheckCaptcha, LemmyContext};

#[async_trait::async_trait(?Send)]
impl PerformCrud for Register {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Register = self;

    let site_view = blocking(context.pool(), SiteView::read_local).await??;
    let local_site = site_view.local_site;

    if !local_site.open_registration {
      return Err(LemmyError::from_message("registration_closed"));
    }

    password_length_check(&data.password)?;
    honeypot_check(&data.honeypot)?;

    if local_site.require_email_verification && data.email.is_none() {
      return Err(LemmyError::from_message("email_required"));
    }

    if local_site.require_application && data.answer.is_none() {
      return Err(LemmyError::from_message(
        "registration_application_answer_required",
      ));
    }

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(LemmyError::from_message("passwords_dont_match"));
    }

    // Check if there are admins. False if admins exist
    let no_admins = blocking(context.pool(), move |conn| {
      PersonViewSafe::admins(conn).map(|a| a.is_empty())
    })
    .await??;

    // If its not the admin, check the captcha
    if !no_admins && local_site.captcha_enabled {
      let check = context
        .chat_server()
        .send(CheckCaptcha {
          uuid: data
            .captcha_uuid
            .to_owned()
            .unwrap_or_else(|| "".to_string()),
          answer: data
            .captcha_answer
            .to_owned()
            .unwrap_or_else(|| "".to_string()),
        })
        .await?;
      if !check {
        return Err(LemmyError::from_message("captcha_incorrect"));
      }
    }

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs(&data.username, &slur_regex)?;
    check_slurs_opt(&data.answer, &slur_regex)?;

    let actor_keypair = generate_actor_keypair()?;
    if !is_valid_actor_name(&data.username, local_site.actor_name_max_length as usize) {
      return Err(LemmyError::from_message("invalid_username"));
    }
    let actor_id = generate_local_apub_endpoint(
      EndpointType::Person,
      &data.username,
      &context.settings().get_protocol_and_hostname(),
    )?;

    // We have to create both a person, and local_user

    // Register the new person
    let person_form = PersonInsertForm::builder()
      .name(data.username.to_owned())
      .actor_id(Some(actor_id.clone()))
      .private_key(Some(actor_keypair.private_key))
      .public_key(actor_keypair.public_key)
      .inbox_url(Some(generate_inbox_url(&actor_id)?))
      .shared_inbox_url(Some(generate_shared_inbox_url(&actor_id)?))
      .admin(Some(no_admins))
      .instance_id(site_view.site.instance_id)
      .build();

    // insert the person
    let inserted_person = blocking(context.pool(), move |conn| {
      Person::create(conn, &person_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "user_already_exists"))?;

    // Create the local user
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .email(data.email.as_deref().map(|s| s.to_lowercase()))
      .password_encrypted(data.password.to_string())
      .show_nsfw(Some(data.show_nsfw))
      .build();

    let inserted_local_user = match blocking(context.pool(), move |conn| {
      LocalUser::create(conn, &local_user_form)
    })
    .await?
    {
      Ok(lu) => lu,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"local_user_email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        // If the local user creation errored, then delete that person
        blocking(context.pool(), move |conn| {
          Person::delete(conn, inserted_person.id)
        })
        .await??;

        return Err(LemmyError::from_error_message(e, err_type));
      }
    };

    if local_site.require_application {
      // Create the registration application
      let form = RegistrationApplicationInsertForm {
        local_user_id: inserted_local_user.id,
        // We already made sure answer was not null above
        answer: data.answer.to_owned().expect("must have an answer"),
      };

      blocking(context.pool(), move |conn| {
        RegistrationApplication::create(conn, &form)
      })
      .await??;
    }

    // Email the admins
    if local_site.application_email_admins {
      send_new_applicant_email_to_admins(&data.username, context.pool(), context.settings())
        .await?;
    }

    let mut login_response = LoginResponse {
      jwt: None,
      registration_created: false,
      verify_email_sent: false,
    };

    // Log the user in directly if email verification and application aren't required
    if !local_site.require_application && !local_site.require_email_verification {
      login_response.jwt = Some(
        Claims::jwt(
          inserted_local_user.id.0,
          &context.secret().jwt_secret,
          &context.settings().hostname,
        )?
        .into(),
      );
    } else {
      if local_site.require_email_verification {
        let local_user_view = LocalUserView {
          local_user: inserted_local_user,
          person: inserted_person,
          counts: PersonAggregates::default(),
        };
        // we check at the beginning of this method that email is set
        let email = local_user_view
          .local_user
          .email
          .clone()
          .expect("email was provided");

        send_verification_email(&local_user_view, &email, context.pool(), context.settings())
          .await?;
        login_response.verify_email_sent = true;
      }

      if local_site.require_application {
        login_response.registration_created = true;
      }
    }

    Ok(login_response)
  }
}
