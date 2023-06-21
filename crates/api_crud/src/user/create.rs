use crate::PerformCrud;
use activitypub_federation::http_signatures::generate_actor_keypair;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, Register},
  utils::{
    generate_inbox_url,
    generate_local_apub_endpoint,
    generate_shared_inbox_url,
    honeypot_check,
    local_site_to_slur_regex,
    password_length_check,
    send_new_applicant_email_to_admins,
    send_verification_email,
    EndpointType,
  },
};
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  source::{
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  traits::Crud,
  RegistrationMode,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  claims::Claims,
  error::LemmyError,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::is_valid_actor_name,
  },
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for Register {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &Register = self;

    let site_view = SiteView::read_local(context.pool()).await?;
    let local_site = site_view.local_site;
    let require_registration_application =
      local_site.registration_mode == RegistrationMode::RequireApplication;

    if local_site.registration_mode == RegistrationMode::Closed {
      return Err(LemmyError::from_message("registration_closed"));
    }

    password_length_check(&data.password)?;
    honeypot_check(&data.honeypot)?;

    if local_site.require_email_verification && data.email.is_none() {
      return Err(LemmyError::from_message("email_required"));
    }

    if local_site.site_setup && require_registration_application && data.answer.is_none() {
      return Err(LemmyError::from_message(
        "registration_application_answer_required",
      ));
    }

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(LemmyError::from_message("passwords_dont_match"));
    }

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs(&data.username, &slur_regex)?;
    check_slurs_opt(&data.answer, &slur_regex)?;

    let actor_keypair = generate_actor_keypair()?;
    is_valid_actor_name(&data.username, local_site.actor_name_max_length as usize)?;
    let actor_id = generate_local_apub_endpoint(
      EndpointType::Person,
      &data.username,
      &context.settings().get_protocol_and_hostname(),
    )?;

    if let Some(email) = &data.email {
      if LocalUser::is_email_taken(context.pool(), email).await? {
        return Err(LemmyError::from_message("email_already_exists"));
      }
    }

    // We have to create both a person, and local_user

    // Register the new person
    let person_form = PersonInsertForm::builder()
      .name(data.username.clone())
      .actor_id(Some(actor_id.clone()))
      .private_key(Some(actor_keypair.private_key))
      .public_key(actor_keypair.public_key)
      .inbox_url(Some(generate_inbox_url(&actor_id)?))
      .shared_inbox_url(Some(generate_shared_inbox_url(&actor_id)?))
      // If its the initial site setup, they are an admin
      .admin(Some(!local_site.site_setup))
      .instance_id(site_view.site.instance_id)
      .build();

    // insert the person
    let inserted_person = Person::create(context.pool(), &person_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "user_already_exists"))?;

    // Automatically set their application as accepted, if they created this with open registration.
    // Also fixes a bug which allows users to log in when registrations are changed to closed.
    let accepted_application = Some(!require_registration_application);

    // Create the local user
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .email(data.email.as_deref().map(str::to_lowercase))
      .password_encrypted(data.password.to_string())
      .show_nsfw(Some(data.show_nsfw))
      .accepted_application(accepted_application)
      .build();

    let inserted_local_user = LocalUser::create(context.pool(), &local_user_form).await?;

    if local_site.site_setup && require_registration_application {
      // Create the registration application
      let form = RegistrationApplicationInsertForm {
        local_user_id: inserted_local_user.id,
        // We already made sure answer was not null above
        answer: data.answer.clone().expect("must have an answer"),
      };

      RegistrationApplication::create(context.pool(), &form).await?;
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

    // Log the user in directly if the site is not setup, or email verification and application aren't required
    if !local_site.site_setup
      || (!require_registration_application && !local_site.require_email_verification)
    {
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

      if require_registration_application {
        login_response.registration_created = true;
      }
    }

    Ok(login_response)
  }
}
