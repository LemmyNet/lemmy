use activitypub_federation::config::Data;
use actix_web::{http::StatusCode, web::Query, HttpRequest, HttpResponse};
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  external_auth::{OAuth, OAuthResponse, TokenResponse},
  utils::create_login_cookie,
};
use lemmy_api_crud::user::create::register_from_oauth;
use lemmy_db_schema::{newtypes::ExternalAuthId, source::local_user::LocalUser, RegistrationMode};
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
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=internal"))
      .finish();
  }

  let state = serde_json::from_str::<OAuthResponse>(&data.state);
  if !state.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=oauth_response"))
      .finish();
  }
  let oauth_state = state.unwrap();

  // Fetch the auth method
  let external_auth_id = ExternalAuthId(oauth_state.external_auth);
  let external_auth_view = ExternalAuthView::get(&mut context.pool(), external_auth_id).await;
  if !external_auth_view.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=external_auth"))
      .finish();
  }
  let external_auth = external_auth_view.unwrap().external_auth;
  let client_secret =
    ExternalAuthView::get_client_secret(&mut context.pool(), external_auth_id).await;
  if !client_secret.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=external_auth"))
      .finish();
  }

  // Get endpoints
  let token_endpoint;
  let user_endpoint;
  if external_auth.auth_type == "oidc" {
    let discovery_endpoint = if external_auth
      .issuer
      .ends_with("/.well-known/openid-configuration")
    {
      external_auth.issuer.to_string()
    } else {
      format!("{}/.well-known/openid-configuration", external_auth.issuer)
    };
    let res = context.client().get(discovery_endpoint).send().await;
    if !res.is_ok() {
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=external_auth"))
        .finish();
    }
    let oidc_response = res.unwrap().json::<serde_json::Value>().await;
    if !oidc_response.is_ok() {
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=external_auth"))
        .finish();
    }
    let oidc_information = oidc_response.unwrap();
    let token_endpoint_string =
      serde_json::from_value::<String>(oidc_information["token_endpoint"].clone());
    if !token_endpoint_string.is_ok() {
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=external_auth"))
        .finish();
    }
    token_endpoint = Url::parse(&token_endpoint_string.unwrap());
    let user_endpoint_string =
      serde_json::from_value::<String>(oidc_information["userinfo_endpoint"].clone());
    if !user_endpoint_string.is_ok() {
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=external_auth"))
        .finish();
    }
    user_endpoint = Url::parse(&user_endpoint_string.unwrap());
  } else {
    token_endpoint = Url::parse(&external_auth.token_endpoint);
    user_endpoint = Url::parse(&external_auth.user_endpoint);
  };
  if !token_endpoint.is_ok() || !user_endpoint.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=external_auth5"))
      .finish();
  }

  // Send token request
  let mut response = context
    .client()
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
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=token"))
      .finish();
  }
  let mut res = response.unwrap();
  if res.status() != StatusCode::OK {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=token"))
      .finish();
  }

  // Obtain access token
  let token_response = res.json::<TokenResponse>().await;
  if !token_response.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=token"))
      .finish();
  }
  let access_token = token_response.unwrap().access_token;

  // Make user info request
  response = context
    .client()
    .post(user_endpoint.unwrap())
    .bearer_auth(access_token)
    .send()
    .await;
  if !response.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=userinfo"))
      .finish();
  }
  res = response.unwrap();
  if res.status() != StatusCode::OK {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=userinfo"))
      .finish();
  }

  // Find or create user
  let userinfo = res.json::<serde_json::Value>().await;
  if !userinfo.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=external_auth"))
      .finish();
  }
  let user_info = userinfo.unwrap();
  let user_id = serde_json::from_value::<String>(user_info["email"].clone());
  if !user_id.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=user"))
      .finish();
  }
  let email = user_id.unwrap();

  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &email).await;
  let local_site = site_view.unwrap().local_site;
  let local_user: LocalUser;
  if local_user_view.is_ok() {
    local_user = local_user_view.unwrap().local_user;
  } else if local_site.oauth_registration {
    let username = serde_json::from_value::<String>(user_info[external_auth.id_attribute].clone());
    if !username.is_ok() {
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=external_auth"))
        .finish();
    }
    let user = str::replace(&username.unwrap(), " ", "_");
    let registered_user = register_from_oauth(user, email, &context).await;
    if !registered_user.is_ok() {
      tracing::error!("Failed to create user: {}", registered_user.err().unwrap());
      return HttpResponse::Found()
        .append_header(("Location", "/login?err=user"))
        .finish();
    }
    local_user = registered_user.unwrap();
  } else {
    return HttpResponse::Found()
      .append_header(("Location", "/signup"))
      .finish();
  }

  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user.accepted_application
    && !local_user.admin
  {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=application"))
      .finish();
  }

  // Check email is verified regardless of site setting, to prevent potential account theft
  if !local_user.admin && !local_user.email_verified {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=email"))
      .finish();
  }

  let jwt = Claims::generate(local_user.id, req, &context).await;
  if !jwt.is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=jwt"))
      .finish();
  }

  let mut res = HttpResponse::build(StatusCode::FOUND)
    .insert_header(("Location", oauth_state.client_redirect_uri))
    .finish();
  let mut cookie = create_login_cookie(jwt.unwrap());
  cookie.set_path("/");
  cookie.set_http_only(false); // We'll need to access the cookie via document.cookie for this req
  if !res.add_cookie(&cookie).is_ok() {
    return HttpResponse::Found()
      .append_header(("Location", "/login?err=jwt"))
      .finish();
  }
  return res;
}
