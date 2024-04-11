use activitypub_federation::{config::Data, http_signatures::generate_actor_keypair};
use actix_web::{web::Json, HttpRequest};
use lemmy_api_common::{
  claims::Claims,
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
    captcha_answer::{CaptchaAnswer, CheckCaptchaAnswer},
    language::Language,
    local_user::{LocalUser, LocalUserInsertForm},
    local_user_vote_display_mode::LocalUserVoteDisplayMode,
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  traits::Crud,
  RegistrationMode,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::is_valid_actor_name,
  },
};
use std::collections::HashSet;

#[tracing::instrument(skip(context))]
pub async fn register(
  data: Json<Register>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let require_registration_application =
    local_site.registration_mode == RegistrationMode::RequireApplication;

  if local_site.registration_mode == RegistrationMode::Closed {
    Err(LemmyErrorType::RegistrationClosed)?
  }

  password_length_check(&data.password)?;
  honeypot_check(&data.honeypot)?;

  if local_site.require_email_verification && data.email.is_none() {
    Err(LemmyErrorType::EmailRequired)?
  }

  if local_site.site_setup && require_registration_application && data.answer.is_none() {
    Err(LemmyErrorType::RegistrationApplicationAnswerRequired)?
  }

  // Make sure passwords match
  if data.password != data.password_verify {
    Err(LemmyErrorType::PasswordsDoNotMatch)?
  }

  if local_site.site_setup && local_site.captcha_enabled {
    if let Some(captcha_uuid) = &data.captcha_uuid {
      let uuid = uuid::Uuid::parse_str(captcha_uuid)?;
      let check = CaptchaAnswer::check_captcha(
        &mut context.pool(),
        CheckCaptchaAnswer {
          uuid,
          answer: data.captcha_answer.clone().unwrap_or_default(),
        },
      )
      .await?;
      if !check {
        Err(LemmyErrorType::CaptchaIncorrect)?
      }
    } else {
      Err(LemmyErrorType::CaptchaIncorrect)?
    }
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
    if LocalUser::is_email_taken(&mut context.pool(), email).await? {
      Err(LemmyErrorType::EmailAlreadyExists)?
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
    .shared_inbox_url(Some(generate_shared_inbox_url(context.settings())?))
    .instance_id(site_view.site.instance_id)
    .build();

  // insert the person
  let inserted_person = Person::create(&mut context.pool(), &person_form)
    .await
    .with_lemmy_type(LemmyErrorType::UserAlreadyExists)?;

  // Automatically set their application as accepted, if they created this with open registration.
  // Also fixes a bug which allows users to log in when registrations are changed to closed.
  let accepted_application = Some(!require_registration_application);

  // Get the user's preferred language using the Accept-Language header
  let language_tags: Vec<String> = req
    .headers()
    .get("Accept-Language")
    .map(|hdr| accept_language::parse(hdr.to_str().unwrap_or_default()))
    .iter()
    .flatten()
    // Remove the optional region code
    .map(|lang_str| lang_str.split('-').next().unwrap_or_default().to_string())
    .collect();

  // Show nsfw content if param is true, or if content_warning exists
  let show_nsfw = data
    .show_nsfw
    .unwrap_or(site_view.site.content_warning.is_some());

  // Create the local user
  let local_user_form = LocalUserInsertForm::builder()
    .person_id(inserted_person.id)
    .email(data.email.as_deref().map(str::to_lowercase))
    .password_encrypted(data.password.to_string())
    .show_nsfw(Some(show_nsfw))
    .accepted_application(accepted_application)
    .default_listing_type(Some(local_site.default_post_listing_type))
    .post_listing_mode(Some(local_site.default_post_listing_mode))
    .interface_language(language_tags.first().cloned())
    // If its the initial site setup, they are an admin
    .admin(Some(!local_site.site_setup))
    .build();

  let all_languages = Language::read_all(&mut context.pool()).await?;
  // use hashset to avoid duplicates
  let mut language_ids = HashSet::new();
  for l in language_tags {
    if let Some(found) = all_languages.iter().find(|all| all.code == l) {
      language_ids.insert(found.id);
    }
  }
  let language_ids = language_ids.into_iter().collect();

  let inserted_local_user =
    LocalUser::create(&mut context.pool(), &local_user_form, language_ids).await?;

  if local_site.site_setup && require_registration_application {
    // Create the registration application
    let form = RegistrationApplicationInsertForm {
      local_user_id: inserted_local_user.id,
      // We already made sure answer was not null above
      answer: data.answer.clone().expect("must have an answer"),
    };

    RegistrationApplication::create(&mut context.pool(), &form).await?;
  }

  // Email the admins, only if email verification is not required
  if local_site.application_email_admins && !local_site.require_email_verification {
    send_new_applicant_email_to_admins(&data.username, &mut context.pool(), context.settings())
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
    let jwt = Claims::generate(inserted_local_user.id, req, &context).await?;
    login_response.jwt = Some(jwt);
  } else {
    if local_site.require_email_verification {
      let local_user_view = LocalUserView {
        local_user: inserted_local_user,
        local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
        person: inserted_person,
        counts: PersonAggregates::default(),
      };
      // we check at the beginning of this method that email is set
      let email = local_user_view
        .local_user
        .email
        .clone()
        .expect("email was provided");

      send_verification_email(
        &local_user_view,
        &email,
        &mut context.pool(),
        context.settings(),
      )
      .await?;
      login_response.verify_email_sent = true;
    }

    if require_registration_application {
      login_response.registration_created = true;
    }
  }

  Ok(Json(login_response))
}
