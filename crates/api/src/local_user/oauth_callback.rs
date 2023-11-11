use actix_web::{
  http::StatusCode,
  web::{Data, Query},
  HttpRequest,
  HttpResponse,
};
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  external_auth::{OAuth, TokenResponse},
  utils::{create_login_cookie},
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, registration_application::RegistrationApplication},
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::{ExternalAuthView, LocalUserView, SiteView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn oauth_callback(
  data: Query<OAuth>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  if !data.state.contains("|") {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  let stateParts = data.state.split("|");
  let client_id = stateParts.next();
  let client_redirect_uri = stateParts.next();

  // Fetch the auth method
  let external_auth = ExternalAuthView::get(&mut context.pool(), ExternalAuthId(client_id.into()))
    .await
    .with_lemmy_type(LemmyErrorType::IncorrectLogin)?
    .external_auth;
  let client_secret = ExternalAuthView::get_client_secret(&mut context.pool(), client_id)
    .await
    .with_lemmy_type(LemmyErrorType::IncorrectLogin)?;

  // Send token request
  let response = context.client()
    .post(external_auth.token_endpoint)
    .form(&[
        ("grant_type", "authorization_code"),
        ("code", data.code),
        ("redirect_uri", req.uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ])
    .send()
    .await?;

  // Check token response
  if req.status != StatusCode::OK {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  // Obtain access token
  let access_token = response.json::<TokenResponse>().await?.access_token;

  // Make user info request
  let response = context.client()
    .post(external_auth.user_endpoint)
    .bearer_auth(access_token)
    .send()
    .await?;

  // Find or create user
  let email = response.json::<serde_json::Value>().await?;
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &email)
      .await;
  if local_user_view.is_ok() {
    // Found user
    check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
        .await?;
    // Check email is verified regardless of site setting, to prevent potential account theft
    if !local_user_view.local_user.admin && !local_user_view.local_user.email_verified {
        Err(LemmyErrorType::EmailNotVerified)?
    }
  } else {
    // TODO register user - how to handle registration applications? show_nsfw? overriding username?
    Err(LemmyErrorType::IncorrectLogin)?
  }

  let jwt = Claims::generate(local_user_view.local_user.id, req, &context).await?;

  let mut res = HttpResponse::build(StatusCode::FOUND)
    .insert_header(("Location", client_redirect_uri))
    .finish();
  res.add_cookie(&create_login_cookie(jwt))?;
  Ok(res)
}

async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user_view.local_user.accepted_application
    && !local_user_view.local_user.admin
  {
    // Fetch the registration application. If no admin id is present its still pending. Otherwise it
    // was processed (either accepted or denied).
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id).await?;
    if registration.admin_id.is_some() {
      Err(LemmyErrorType::RegistrationDenied(registration.deny_reason))?
    } else {
      Err(LemmyErrorType::RegistrationApplicationIsPending)?
    }
  }
  Ok(())
}
