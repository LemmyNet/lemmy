use activitypub_federation::{config::Data};
use actix_web::{
  http::StatusCode,
  web::Query,
  HttpRequest,
  HttpResponse,
};
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  external_auth::{OAuth, OAuthResponse, TokenResponse},
  utils::{create_login_cookie},
};
use lemmy_api_crud::user::create::register_from_oauth;
use lemmy_db_schema::{
  newtypes::ExternalAuthId,
  RegistrationMode,
  source::local_user::LocalUser,
};
use lemmy_db_views::structs::{ExternalAuthView, LocalUserView, SiteView};
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn oauth_callback(
  data: Query<OAuth>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> HttpResponse {
  let site_view = SiteView::read_local(&mut context.pool()).await;

  if !site_view.is_ok() {    
    return HttpResponse::Found().append_header(("Location", "/login?err=internal")).finish();
  }

  let state = serde_json::from_str::<OAuthResponse>(&data.state);
  if !state.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=oauth_response")).finish();
  }
  let oauth_state = state.unwrap();

  // Fetch the auth method
  let external_auth_id = ExternalAuthId(oauth_state.external_auth);
  let external_auth_view = ExternalAuthView::get(&mut context.pool(), external_auth_id).await;
  if !external_auth_view.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
  }
  let external_auth = external_auth_view.unwrap().external_auth;
  let client_secret = ExternalAuthView::get_client_secret(&mut context.pool(), external_auth_id)
    .await;
  if !client_secret.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
  }

  // Send token request
  let token_endpoint = Url::parse(&external_auth.token_endpoint);
  if !token_endpoint.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
  }
  let mut response = context.client()
    .post(token_endpoint.unwrap())
    .form(&[
        ("grant_type", "authorization_code"),
        ("code", &data.code),
        ("redirect_uri", &req.uri().to_string()),
        ("client_id", &external_auth.client_id),
        ("client_secret", &client_secret.unwrap()),
    ])
    .send()
    .await;
  if !response.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=token")).finish();
  }
  let mut res = response.unwrap();
  if res.status() != StatusCode::OK {
    return HttpResponse::Found().append_header(("Location", "/login?err=token")).finish();
  }

  // Obtain access token
  let token_response = res.json::<TokenResponse>().await;
  if !token_response.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=token")).finish();
  }
  let access_token = token_response.unwrap().access_token;

  // Make user info request
  let user_endpoint = Url::parse(&external_auth.user_endpoint);
  if !user_endpoint.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
  }
  response = context.client()
    .post(user_endpoint.unwrap())
    .bearer_auth(access_token)
    .send()
    .await;
  if !response.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=userinfo")).finish();
  }
  res = response.unwrap();
  if res.status() != StatusCode::OK {
    return HttpResponse::Found().append_header(("Location", "/login?err=userinfo")).finish();
  }

  // Find or create user
  let userinfo = res.json::<serde_json::Value>().await;
  if !userinfo.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
  }
  let user_info = userinfo.unwrap();
  let user_id = serde_json::from_value::<String>(user_info["email"].clone());
  if !user_id.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=user")).finish();
  }
  let email = user_id.unwrap();

  let local_user_view =
    LocalUserView::find_by_email(&mut context.pool(), &email).await;
  let local_site = site_view.unwrap().local_site;
  let local_user: LocalUser;
  if local_user_view.is_ok() {
    local_user = local_user_view.unwrap().local_user;
  } else {
    let username = serde_json::from_value::<String>(user_info[external_auth.id_attribute]
      .clone());
    if !username.is_ok() {
      return HttpResponse::Found().append_header(("Location", "/login?err=external_auth")).finish();
    }
    let registered_user = register_from_oauth(username.unwrap(), email, &context).await;
    if !registered_user.is_ok() {
      return HttpResponse::Found().append_header(("Location", "/login?err=user")).finish();
    }
    local_user = registered_user.unwrap();

    // if registration is not allowed
    // return HttpResponse::Found().append_header(("Location", "/signup")).finish();
  }

  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user.accepted_application
    && !local_user.admin {
    return HttpResponse::Found().append_header(("Location", "/login?err=application")).finish();
  }

  // Check email is verified regardless of site setting, to prevent potential account theft
  if !local_user.admin && !local_user.email_verified {
    return HttpResponse::Found().append_header(("Location", "/login?err=email")).finish();
  }

  let jwt = Claims::generate(local_user.id, req, &context).await;
  if !jwt.is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=jwt")).finish();
  }

  let mut res = HttpResponse::build(StatusCode::FOUND)
    .insert_header(("Location", oauth_state.client_redirect_uri))
    .finish();
  if !res.add_cookie(&create_login_cookie(jwt.unwrap())).is_ok() {
    return HttpResponse::Found().append_header(("Location", "/login?err=jwt")).finish();
  }
  return res;  
}
