use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  http_signatures::generate_actor_keypair,
};
use actix_web::{HttpRequest, rt::time::sleep, web::Json};
use diesel_async::{AsyncPgConnection, scoped_futures::ScopedFutureExt};
use lemmy_api_utils::{
  claims::Claims,
  context::LemmyContext,
  utils::{
    check_email_verified,
    check_local_user_valid,
    check_registration_application,
    generate_featured_url,
    generate_followers_url,
    generate_inbox_url,
    generate_moderators_url,
    honeypot_check,
    password_length_check,
    slur_regex,
  },
};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::{
  newtypes::OAuthProviderId,
  source::{
    actor_language::SiteLanguage,
    captcha_answer::{CaptchaAnswer, CheckCaptchaAnswer},
    community::{Community, CommunityActions, CommunityInsertForm, CommunityModeratorForm},
    language::Language,
    local_site::LocalSite,
    local_user::{LocalUser, LocalUserInsertForm},
    oauth_account::{OAuthAccount, OAuthAccountInsertForm},
    oauth_provider::AdminOAuthProvider,
    person::{Person, PersonInsertForm},
    post::{Post, PostActions, PostInsertForm, PostLikeForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  traits::{ApubActor, Likeable},
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_registration_applications::api::Register;
use lemmy_db_views_site::{
  SiteView,
  api::{AuthenticateWithOauth, LoginResponse},
};
use lemmy_diesel_utils::{connection::get_conn, pagination::PagedResponse, traits::Crud};
use lemmy_email::{
  account::send_verification_email_if_required,
  admin::send_new_applicant_email_to_admins,
  user_language,
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  spawn_try_task,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::is_valid_actor_name,
  },
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashSet, sync::LazyLock, time::Duration};
use tracing::info;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Response from OAuth token endpoint
struct TokenResponse {
  pub access_token: String,
  pub token_type: String,
  pub expires_in: Option<i64>,
  pub refresh_token: Option<String>,
  pub scope: Option<String>,
}

pub async fn register(
  Json(data): Json<Register>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponse>> {
  let pool = &mut context.pool();
  let site_view = SiteView::read_local(pool).await?;
  let local_site = site_view.local_site.clone();
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

  // make sure the registration answer is provided when the registration application is required
  if local_site.site_setup {
    validate_registration_answer(require_registration_application, &data.answer)?;
  }

  // Make sure passwords match
  if data.password != data.password_verify {
    Err(LemmyErrorType::PasswordsDoNotMatch)?
  }

  if local_site.site_setup && local_site.captcha_enabled {
    let uuid = uuid::Uuid::parse_str(&data.captcha_uuid.clone().unwrap_or_default())?;
    CaptchaAnswer::check_captcha(
      pool,
      CheckCaptchaAnswer {
        uuid,
        answer: data.captcha_answer.clone().unwrap_or_default(),
      },
    )
    .await?;
  }

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.username, &slur_regex)?;
  check_slurs_opt(&data.answer, &slur_regex)?;

  Person::check_username_taken(pool, &data.username).await?;

  if let Some(email) = &data.email {
    LocalUser::check_is_email_taken(pool, email).await?;
  }

  // Automatically set their application as accepted, if they created this with open registration.
  // Also fixes a bug which allows users to log in when registrations are changed to closed.
  let accepted_application = Some(!require_registration_application);

  // Show nsfw content if param is true, or if content_warning exists
  let show_nsfw = data
    .show_nsfw
    .unwrap_or(site_view.site.content_warning.is_some());

  let language_tags = get_language_tags(&req);

  // Wrap the insert person, insert local user, and create registration,
  // in a transaction, so that if any fail, the rows aren't created.
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  let tx_context = context.clone();
  let user = conn
    .run_transaction(|conn| {
      async move {
        // We have to create both a person, and local_user
        let person = create_person(tx_data.username.clone(), &site_view, &tx_context, conn).await?;

        // Create the local user
        let local_user_form = LocalUserInsertForm {
          email: tx_data.email.as_deref().map(str::to_lowercase),
          show_nsfw: Some(show_nsfw),
          accepted_application,
          ..LocalUserInsertForm::new(person.id, Some(tx_data.password.to_string()))
        };

        let local_user = create_local_user(
          conn,
          language_tags,
          local_user_form,
          &site_view.local_site,
          &tx_context,
        )
        .await?;

        if site_view.local_site.site_setup
          && require_registration_application
          && let Some(answer) = tx_data.answer.clone()
        {
          // Create the registration application
          let form = RegistrationApplicationInsertForm {
            local_user_id: local_user.id,
            answer,
          };

          RegistrationApplication::create(&mut conn.into(), &form).await?;
        }

        Ok(LocalUserView {
          person,
          local_user,
          banned: false,
          ban_expires_at: None,
        })
      }
      .scope_boxed()
    })
    .await?;

  // Email the admins, only if email verification is not required
  if local_site.application_email_admins && !local_site.require_email_verification {
    send_new_applicant_email_to_admins(&data.username, pool, context.settings()).await?;
  }

  let mut login_response = LoginResponse {
    jwt: None,
    registration_created: false,
    verify_email_sent: false,
  };

  // Log the user in directly if the site is not setup, or email verification and application aren't
  // required
  if !local_site.site_setup
    || (!require_registration_application && !local_site.require_email_verification)
  {
    let jwt = Claims::generate(user.local_user.id, data.stay_logged_in, req, &context).await?;
    login_response.jwt = Some(jwt);
  } else {
    login_response.verify_email_sent = send_verification_email_if_required(
      &local_site,
      &user,
      &mut context.pool(),
      context.settings(),
    )
    .await?;

    if require_registration_application {
      login_response.registration_created = true;
    }
  }

  Ok(Json(login_response))
}

pub async fn authenticate_with_oauth(
  Json(data): Json<AuthenticateWithOauth>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponse>> {
  let pool = &mut context.pool();
  let site_view = SiteView::read_local(pool).await?;
  let local_site = site_view.local_site.clone();

  // Show nsfw content if param is true, or if content_warning exists
  let show_nsfw = data
    .show_nsfw
    .unwrap_or(site_view.site.content_warning.is_some());

  let language_tags = get_language_tags(&req);

  // validate inputs
  if data.oauth_provider_id == OAuthProviderId(0) || data.code.is_empty() || data.code.len() > 300 {
    return Err(LemmyErrorType::OauthAuthorizationInvalid)?;
  }

  // validate the redirect_uri
  let redirect_uri = &data.redirect_uri;
  if redirect_uri.host_str().unwrap_or("").is_empty()
    || !redirect_uri.path().eq(&String::from("/oauth/callback"))
    || !redirect_uri.query().unwrap_or("").is_empty()
  {
    Err(LemmyErrorType::OauthAuthorizationInvalid)?
  }

  // validate the PKCE challenge
  if let Some(code_verifier) = &data.pkce_code_verifier {
    check_code_verifier(code_verifier)?;
  }

  // Fetch the OAUTH provider and make sure it's enabled
  let oauth_provider_id = data.oauth_provider_id;
  let oauth_provider = AdminOAuthProvider::read(pool, oauth_provider_id)
    .await
    .ok()
    .ok_or(LemmyErrorType::OauthAuthorizationInvalid)?;

  if !oauth_provider.enabled {
    return Err(LemmyErrorType::OauthAuthorizationInvalid)?;
  }

  let token_response = oauth_request_access_token(
    &context,
    &oauth_provider,
    &data.code,
    data.pkce_code_verifier.as_deref(),
    redirect_uri.as_str(),
  )
  .await?;

  let user_info = oidc_get_user_info(
    &context,
    &oauth_provider,
    token_response.access_token.as_str(),
  )
  .await?;

  let oauth_user_id = read_user_info(&user_info, oauth_provider.id_claim.as_str())?;

  let require_registration_application =
    local_site.registration_mode == RegistrationMode::RequireApplication;

  let mut login_response = LoginResponse {
    jwt: None,
    registration_created: false,
    verify_email_sent: false,
  };

  // Lookup user by oauth_user_id
  let mut local_user_view =
    LocalUserView::find_by_oauth_id(pool, oauth_provider.id, &oauth_user_id).await;

  let local_user = if let Ok(user_view) = local_user_view {
    // user found by oauth_user_id => Login user
    let local_user = user_view.clone().local_user;

    login_response.registration_created = local_site.site_setup
      && require_registration_application
      && !local_user.accepted_application
      && !local_user.admin
      && data.answer.is_some();

    check_local_user_valid(&user_view)?;
    check_email_verified(&user_view, &site_view)?;
    check_registration_application(&user_view, &site_view.local_site, pool).await?;
    local_user
  } else {
    // user has never previously registered using oauth

    // prevent registration if registration is closed
    if local_site.registration_mode == RegistrationMode::Closed {
      Err(LemmyErrorType::RegistrationClosed)?
    }

    // prevent registration if registration is closed for OAUTH providers
    if !local_site.oauth_registration {
      return Err(LemmyErrorType::OauthRegistrationClosed)?;
    }

    // Extract the OAUTH email claim from the returned user_info
    let email = read_user_info(&user_info, "email")?;

    // Lookup user by OAUTH email and link accounts
    local_user_view = LocalUserView::find_by_email(pool, &email).await;

    if let Ok(user_view) = local_user_view {
      // user found by email => link and login if linking is allowed

      // we only allow linking by email when email_verification is required otherwise emails cannot
      // be trusted
      if oauth_provider.account_linking_enabled && site_view.local_site.require_email_verification {
        // WARNING:
        // If an admin switches the require_email_verification config from false to true,
        // users who signed up before the switch could have accounts with unverified emails falsely
        // marked as verified.

        check_local_user_valid(&user_view)?;
        check_email_verified(&user_view, &site_view)?;
        check_registration_application(&user_view, &site_view.local_site, pool).await?;

        // Link with OAUTH => Login user
        let oauth_account_form =
          OAuthAccountInsertForm::new(user_view.local_user.id, oauth_provider.id, oauth_user_id);

        OAuthAccount::create(pool, &oauth_account_form).await?;

        user_view.local_user.clone()
      } else {
        return Err(LemmyErrorType::EmailAlreadyTaken)?;
      }
    } else {
      // No user was found by email => Register as new user

      // make sure the registration answer is provided when the registration application is required
      validate_registration_answer(require_registration_application, &data.answer)?;

      let slur_regex = slur_regex(&context).await?;

      // Wrap the insert person, insert local user, and create registration,
      // in a transaction, so that if any fail, the rows aren't created.
      let conn = &mut get_conn(pool).await?;
      let tx_data = data.clone();
      let tx_context = context.clone();
      let user = conn
        .run_transaction(|conn| {
          async move {
            // make sure the username is provided
            let username = tx_data
              .username
              .as_ref()
              .ok_or(LemmyErrorType::RegistrationUsernameRequired)?;

            check_slurs(username, &slur_regex)?;
            check_slurs_opt(&tx_data.answer, &slur_regex)?;

            Person::check_username_taken(&mut conn.into(), username).await?;

            // We have to create a person, a local_user, and an oauth_account
            let person = create_person(username.clone(), &site_view, &tx_context, conn).await?;

            // Create the local user
            let local_user_form = LocalUserInsertForm {
              email: Some(str::to_lowercase(&email)),
              show_nsfw: Some(show_nsfw),
              accepted_application: Some(!require_registration_application),
              email_verified: Some(oauth_provider.auto_verify_email),
              ..LocalUserInsertForm::new(person.id, None)
            };

            let local_user = create_local_user(
              conn,
              language_tags,
              local_user_form,
              &site_view.local_site,
              &tx_context,
            )
            .await?;

            // Create the oauth account
            let oauth_account_form =
              OAuthAccountInsertForm::new(local_user.id, oauth_provider.id, oauth_user_id);

            OAuthAccount::create(&mut conn.into(), &oauth_account_form).await?;

            // prevent sign in until application is accepted
            if login_response.registration_created {
              // Create the registration application
              RegistrationApplication::create(
                &mut conn.into(),
                &RegistrationApplicationInsertForm {
                  local_user_id: local_user.id,
                  // We already check earlier that this Some, however using `ok_or` is cleaner
                  // than unwrap or expect (which also requires clippy allow).
                  answer: data
                    .answer
                    .ok_or(LemmyErrorType::RegistrationApplicationAnswerRequired)?,
                },
              )
              .await?;
            }
            Ok(LocalUserView {
              person,
              local_user,
              banned: false,
              ban_expires_at: None,
            })
          }
          .scope_boxed()
        })
        .await?;

      // Check email is verified when required
      login_response.verify_email_sent = send_verification_email_if_required(
        &local_site,
        &user,
        &mut context.pool(),
        context.settings(),
      )
      .await?;
      user.local_user
    }
  };

  if !login_response.registration_created && !login_response.verify_email_sent {
    let jwt = Claims::generate(local_user.id, data.stay_logged_in, req, &context).await?;
    login_response.jwt = Some(jwt);
  }

  Ok(Json(login_response))
}

async fn create_person(
  username: String,
  site_view: &SiteView,
  context: &LemmyContext,
  conn: &mut AsyncPgConnection,
) -> Result<Person, LemmyError> {
  let actor_keypair = generate_actor_keypair()?;
  is_valid_actor_name(&username)?;
  let ap_id = Person::generate_local_actor_url(&username, context.settings())?;

  // Register the new person
  let person_form = PersonInsertForm {
    ap_id: Some(ap_id.clone()),
    inbox_url: Some(generate_inbox_url()?),
    private_key: Some(actor_keypair.private_key),
    ..PersonInsertForm::new(
      username.clone(),
      actor_keypair.public_key,
      site_view.site.instance_id,
    )
  };

  // insert the person
  let inserted_person = Person::create(&mut conn.into(), &person_form).await?;

  Ok(inserted_person)
}

fn get_language_tags(req: &HttpRequest) -> Vec<String> {
  req
    .headers()
    .get("Accept-Language")
    .map(|hdr| accept_language::parse(hdr.to_str().unwrap_or_default()))
    .iter()
    .flatten()
    // Remove the optional region code
    .map(|lang_str| lang_str.split('-').next().unwrap_or_default().to_string())
    .collect::<Vec<String>>()
}

async fn create_local_user(
  conn: &mut AsyncPgConnection,
  language_tags: Vec<String>,
  mut local_user_form: LocalUserInsertForm,
  local_site: &LocalSite,
  context: &Data<LemmyContext>,
) -> Result<LocalUser, LemmyError> {
  let conn_ = &mut conn.into();
  let all_languages = Language::read_all(conn_).await?;
  // use hashset to avoid duplicates
  let mut language_ids = HashSet::new();

  // Enable site languages. Ignored if all languages are enabled.
  let discussion_languages = SiteLanguage::read(conn_, local_site.site_id).await?;

  // Enable languages from `Accept-Language` header only if no site languages are set. Otherwise it
  // is possible that browser languages are only set to e.g. French, and the user won't see any
  // English posts.
  if !discussion_languages.is_empty() {
    for l in &language_tags {
      if let Some(found) = all_languages.iter().find(|all| &all.code == l) {
        language_ids.insert(found.id);
      }
    }
  }
  language_ids.extend(discussion_languages);

  let language_ids = language_ids.into_iter().collect();

  local_user_form.default_listing_type = Some(local_site.default_post_listing_type);
  local_user_form.post_listing_mode = Some(local_site.default_post_listing_mode);
  // If its the initial site setup, they are an admin
  local_user_form.admin = Some(!local_site.site_setup);
  local_user_form.interface_language = language_tags.first().cloned();
  let inserted_local_user = LocalUser::create(conn_, &local_user_form, language_ids).await?;

  // If we are setting up a new site, fetch initial communities and create welcome post.
  if !local_site.site_setup {
    local_user_form.admin = Some(true);
    create_welcome_post(inserted_local_user.clone(), context);
    fetch_community_list(context.clone());
  }

  Ok(inserted_local_user)
}

fn validate_registration_answer(
  require_registration_application: bool,
  answer: &Option<String>,
) -> LemmyResult<()> {
  if require_registration_application && answer.is_none() {
    Err(LemmyErrorType::RegistrationApplicationAnswerRequired)?
  }

  Ok(())
}

async fn oauth_request_access_token(
  context: &Data<LemmyContext>,
  oauth_provider: &AdminOAuthProvider,
  code: &str,
  pkce_code_verifier: Option<&str>,
  redirect_uri: &str,
) -> LemmyResult<TokenResponse> {
  let mut form = vec![
    ("client_id", &*oauth_provider.client_id),
    ("client_secret", &*oauth_provider.client_secret),
    ("code", code),
    ("grant_type", "authorization_code"),
    ("redirect_uri", redirect_uri),
  ];

  if let Some(code_verifier) = pkce_code_verifier {
    form.push(("code_verifier", code_verifier));
  }

  // Request an Access Token from the OAUTH provider
  let response = context
    .client()
    .post(oauth_provider.token_endpoint.as_str())
    .header("Accept", "application/json")
    .form(&form[..])
    .send()
    .await
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?
    .error_for_status()
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?;

  // Extract the access token
  let token_response = response
    .json::<TokenResponse>()
    .await
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?;

  Ok(token_response)
}

async fn oidc_get_user_info(
  context: &Data<LemmyContext>,
  oauth_provider: &AdminOAuthProvider,
  access_token: &str,
) -> LemmyResult<serde_json::Value> {
  // Request the user info from the OAUTH provider
  let response = context
    .client()
    .get(oauth_provider.userinfo_endpoint.as_str())
    .header("Accept", "application/json")
    .bearer_auth(access_token)
    .send()
    .await
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?
    .error_for_status()
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?;

  // Extract the OAUTH user_id claim from the returned user_info
  let user_info = response
    .json::<serde_json::Value>()
    .await
    .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?;

  Ok(user_info)
}

fn read_user_info(user_info: &serde_json::Value, key: &str) -> LemmyResult<String> {
  if let Some(value) = user_info.get(key) {
    let result = serde_json::from_value::<String>(value.clone())
      .with_lemmy_type(LemmyErrorType::OauthLoginFailed)?;
    return Ok(result);
  }
  Err(LemmyErrorType::OauthLoginFailed)?
}

#[allow(clippy::expect_used)]
fn check_code_verifier(code_verifier: &str) -> LemmyResult<()> {
  static VALID_CODE_VERIFIER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9\-._~]{43,128}$").expect("compile regex"));

  let check = VALID_CODE_VERIFIER_REGEX.is_match(code_verifier);

  if check {
    Ok(())
  } else {
    Err(LemmyErrorType::InvalidCodeVerifier.into())
  }
}

fn fetch_community_list(context: Data<LemmyContext>) {
  // Only do this in release mode.
  if cfg!(debug_assertions) {
    //return;
  }

  spawn_try_task(async move {
    let instances = context
      .settings()
      .setup
      .clone()
      .unwrap_or_default()
      .bootstrap_instances;
    let mut communities: Vec<ObjectId<ApubCommunity>> = vec![];
    for i in instances {
      info!("Trying to fetch community list from {i}");
      let res = context
        .client()
        .get(format!(
          "https://{i}/api/v4/community/list?type_=all&sort=active_monthly&limit=50"
        ))
        .send()
        .await;
      if let Ok(res) = res
        && let Ok(json) = res.json::<PagedResponse<CommunityView>>().await
      {
        communities = json
          .items
          .into_iter()
          // exclude nsfw
          .filter(|c| !c.community.nsfw)
          .map(|c| c.community.ap_id.into())
          .collect();
        info!("Successfully fetched community list from {i}");
        break;
      }
      info!("Failed to fetch community list from {i}");
    }
    // also prefetch these two communities as they are linked in the welcome post
    communities.insert(0, "https://lemmy.ml/c/announcements".parse()?);
    communities.insert(0, "https://lemmy.ml/c/lemmy".parse()?);

    // Fetch communities themselves
    let tasks = communities.iter().map(|c| async {
      let context = context.reset_request_count();
      c.dereference(&context).await.ok();
    });

    // This could be made faster by running tasks in parallel with try_join_all or
    // FuturesUnordered. However that causes massive slowdown as each community fetch
    // starts additional background tasks to fetch moderators, recent posts etc. So we
    // need to run it one by one and sleep in between.
    for t in tasks {
      t.await;
      sleep(Duration::from_secs(1)).await;
    }

    Ok(())
  })
}

fn create_welcome_post(local_user: LocalUser, context: &LemmyContext) {
  let context = context.clone();

  spawn_try_task(async move {
    let pool = &mut context.pool();
    let site = SiteView::read_local(pool).await?;
    let admins = PersonView::list_admins(None, site.instance.id, &mut context.pool()).await?;
    let initial_user = admins.first();

    let person = SiteView::read_system_account(&mut context.pool()).await?;

    // Create main community
    let community_name = "main".to_string();
    let community_ap_id = Community::generate_local_actor_url(&community_name, context.settings())?;
    let keypair = generate_actor_keypair()?;
    let community_form = CommunityInsertForm {
      ap_id: Some(community_ap_id.clone()),
      private_key: Some(keypair.private_key),
      followers_url: Some(generate_followers_url(&community_ap_id)?),
      inbox_url: Some(generate_inbox_url()?),
      moderators_url: Some(generate_moderators_url(&community_ap_id)?),
      featured_url: Some(generate_featured_url(&community_ap_id)?),
      ..CommunityInsertForm::new(
        site.site.instance_id,
        community_name,
        "Main".to_string(),
        keypair.public_key,
      )
    };
    let community = Community::create(pool, &community_form).await?;

    // Add initial admin user as community mod (not necessary but looks cleaner)
    if let Some(initial_user) = initial_user {
      let mod_form = CommunityModeratorForm::new(community.id, initial_user.person.id);
      CommunityActions::join(pool, &mod_form).await?;
    }

    // Create post in this community with getting started info
    let lang = user_language(&local_user);
    let title = lang.welcome_post_title().to_string();
    let body = lang.welcome_post_body().to_string();
    let post_form = PostInsertForm {
      body: Some(body),
      featured_local: Some(true),
      ..PostInsertForm::new(title, person.id, community.id)
    };
    let post = Post::create(pool, &post_form).await?;

    // Own upvote for post
    let like_form = PostLikeForm::new(post.id, person.id, Some(true));
    PostActions::like(&mut context.pool(), &like_form).await?;

    Ok(())
  })
}
