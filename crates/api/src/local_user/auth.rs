use actix_web::{
  cookie::{time::Duration, Cookie, SameSite},
  web::{Data, Json},
  HttpRequest,
  HttpResponse,
};
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AccessTokenResponse, GetAccessToken, GetRefreshToken},
  utils::{check_registration_application, check_user_valid},
};
use lemmy_db_schema::{
  source::{
    auth_api_token::{AuthApiToken, AuthApiTokenUpdateForm},
    auth_refresh_token::{
      AuthRefreshToken,
      AuthRefreshTokenCreateForm,
      AuthRefreshTokenUpdateForm,
    },
  },
  utils::naive_now,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  claims::{AuthMethod, Claims},
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::validation::check_totp_2fa_valid,
};

const REFRESH_TOKEN_EXPIRY_WEEKS: i64 = 2;

pub async fn auth_access_token(
  req: HttpRequest,
  data: Json<GetAccessToken>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let local_user_id = match req.cookie("refresh_token") {
    Some(cookie) => {
      let refresh_token =
        AuthRefreshToken::read_from_token(&mut context.pool(), cookie.value()).await?;
      if (naive_now() - refresh_token.last_used).num_weeks() >= REFRESH_TOKEN_EXPIRY_WEEKS {
        return Err(LemmyErrorType::TokenNotFound)?;
      }
      refresh_token.local_user_id
    }
    None => {
      // If no refresh_token cookie exists, then assume this request is made with an api token
      let token = data
        .api_token
        .as_ref()
        .expect("No api_token or refresh_token provided");

      let api_token = AuthApiToken::read_from_token(&mut context.pool(), token).await?;
      if naive_now() > api_token.expires {
        return Err(LemmyErrorType::TokenNotFound)?;
      }
      api_token.local_user_id
    }
  };

  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;

  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

  let mut response_builder = HttpResponse::Ok();

  let auth_method = match req.cookie("refresh_token") {
    Some(mut cookie) => {
      // Update refresh token & cookie
      let token_update_form = AuthRefreshTokenUpdateForm {
        last_used: naive_now(),
        last_ip: req
          .connection_info()
          .realip_remote_addr()
          .unwrap()
          .to_string(),
      };

      AuthRefreshToken::update_token(&mut context.pool(), cookie.value(), &token_update_form)
        .await?;
      cookie.set_max_age(Duration::weeks(REFRESH_TOKEN_EXPIRY_WEEKS));
      response_builder.cookie(cookie);
      AuthMethod::Password
    }
    None => {
      // Update api token
      let token_update_form = AuthApiTokenUpdateForm {
        last_used: naive_now(),
        last_ip: req
          .connection_info()
          .realip_remote_addr()
          .unwrap()
          .to_string(),
      };

      AuthApiToken::update_token(
        &mut context.pool(),
        data.api_token.as_ref().unwrap(),
        &token_update_form,
      )
      .await?;
      AuthMethod::Api
    }
  };

  response_builder.json(AccessTokenResponse {
    jwt: Claims::jwt_with_exp(
      local_user_view.local_user.id.0,
      &context.secret().jwt_secret,
      &context.settings().hostname,
      auth_method,
    )?
    .into(),
  });

  Ok(response_builder.finish())
}

pub async fn auth_refresh_token(
  req: HttpRequest,
  data: Json<GetRefreshToken>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  // Fetch that username / email
  let username_or_email = data.username_or_email.clone();
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &username_or_email)
      .await
      .with_lemmy_type(LemmyErrorType::IncorrectLogin)?;

  // Verify the password
  let valid: bool = verify(
    &data.password,
    &local_user_view.local_user.password_encrypted,
  )
  .unwrap_or(false);
  if !valid {
    return Err(LemmyErrorType::IncorrectLogin)?;
  }
  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

  // Check if the user's email is verified if email verification is turned on
  // However, skip checking verification if the user is an admin
  if !local_user_view.person.admin
    && site_view.local_site.require_email_verification
    && !local_user_view.local_user.email_verified
  {
    return Err(LemmyErrorType::EmailNotVerified)?;
  }

  check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
    .await?;

  // Check the totp
  check_totp_2fa_valid(
    &local_user_view.local_user.totp_2fa_secret,
    &data.totp_2fa_token,
    &site_view.site.name,
    &local_user_view.person.name,
  )?;

  // Create refresh token
  let form = AuthRefreshTokenCreateForm {
    local_user_id: local_user_view.local_user.id,
    last_ip: req
      .connection_info()
      .realip_remote_addr()
      .unwrap()
      .to_string(),
  };
  let refresh_token = AuthRefreshToken::create(&mut context.pool(), &form).await?;

  let cookie = Cookie::build("refresh_token", refresh_token.token)
    .same_site(SameSite::Strict)
    .max_age(Duration::weeks(REFRESH_TOKEN_EXPIRY_WEEKS))
    .path("/api/v3/access_token") // This can only be used for getting access tokens
    .secure(true)
    .http_only(true)
    .finish();

  Ok(HttpResponse::Ok().cookie(cookie).finish())
}
