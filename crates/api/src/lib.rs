use activitypub_federation::config::Data;
use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine};
use captcha::Captcha;
use lemmy_api_common::{
  claims::Claims,
  community::BanFromCommunity,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_expire_time, check_user_valid, local_site_to_slur_regex, AUTH_COOKIE_NAME},
};
use lemmy_db_schema::{
  source::{
    community::{
      CommunityFollower,
      CommunityFollowerForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    local_site::LocalSite,
    moderator::{ModBanFromCommunity, ModBanFromCommunityForm},
    person::Person,
  },
  traits::{Bannable, Crud, Followable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorExt2, LemmyErrorType, LemmyResult},
  utils::slurs::check_slurs,
};
use std::io::Cursor;
use totp_rs::{Secret, TOTP};

pub mod comment;
pub mod comment_report;
pub mod community;
pub mod local_user;
pub mod post;
pub mod post_report;
pub mod private_message;
pub mod private_message_report;
pub mod site;
pub mod sitemap;

/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> Result<String, LemmyError> {
  let letters = captcha.as_wav();

  // Decode each wav file, concatenate the samples
  let mut concat_samples: Vec<i16> = Vec::new();
  let mut any_header: Option<wav::Header> = None;
  for letter in letters {
    let mut cursor = Cursor::new(letter.unwrap_or_default());
    let (header, samples) = wav::read(&mut cursor)?;
    any_header = Some(header);
    if let Some(samples16) = samples.as_sixteen() {
      concat_samples.extend(samples16);
    } else {
      Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
    }
  }

  // Encode the concatenated result as a wav file
  let mut output_buffer = Cursor::new(vec![]);
  if let Some(header) = any_header {
    wav::write(
      header,
      &wav::BitDepth::Sixteen(concat_samples),
      &mut output_buffer,
    )
    .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;

    Ok(base64.encode(output_buffer.into_inner()))
  } else {
    Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
  }
}

/// Check size of report
pub(crate) fn check_report_reason(reason: &str, local_site: &LocalSite) -> Result<(), LemmyError> {
  let slur_regex = &local_site_to_slur_regex(local_site);

  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(LemmyErrorType::ReportReasonRequired)?
  } else if reason.chars().count() > 1000 {
    Err(LemmyErrorType::ReportTooLong)?
  } else {
    Ok(())
  }
}

pub fn read_auth_token(req: &HttpRequest) -> Result<Option<String>, LemmyError> {
  // Try reading jwt from auth header
  if let Ok(header) = Authorization::<Bearer>::parse(req) {
    Ok(Some(header.as_ref().token().to_string()))
  }
  // If that fails, try to read from cookie
  else if let Some(cookie) = &req.cookie(AUTH_COOKIE_NAME) {
    Ok(Some(cookie.value().to_string()))
  }
  // Otherwise, there's no auth
  else {
    Ok(None)
  }
}

pub(crate) fn check_totp_2fa_valid(
  local_user_view: &LocalUserView,
  totp_token: &Option<String>,
  site_name: &str,
) -> LemmyResult<()> {
  // Throw an error if their token is missing
  let token = totp_token
    .as_deref()
    .ok_or(LemmyErrorType::MissingTotpToken)?;
  let secret = local_user_view
    .local_user
    .totp_2fa_secret
    .as_deref()
    .ok_or(LemmyErrorType::MissingTotpSecret)?;

  let totp = build_totp_2fa(site_name, &local_user_view.person.name, secret)?;

  let check_passed = totp.check_current(token)?;
  if !check_passed {
    return Err(LemmyErrorType::IncorrectTotpToken.into());
  }

  Ok(())
}

pub(crate) fn generate_totp_2fa_secret() -> String {
  Secret::generate_secret().to_string()
}

pub(crate) fn build_totp_2fa(
  site_name: &str,
  username: &str,
  secret: &str,
) -> Result<TOTP, LemmyError> {
  let sec = Secret::Raw(secret.as_bytes().to_vec());
  let sec_bytes = sec
    .to_bytes()
    .map_err(|_| LemmyErrorType::CouldntParseTotpSecret)?;

  TOTP::new(
    totp_rs::Algorithm::SHA1,
    6,
    1,
    30,
    sec_bytes,
    Some(site_name.to_string()),
    username.to_string(),
  )
  .with_lemmy_type(LemmyErrorType::CouldntGenerateTotp)
}

/// Site bans are only federated for local users.
/// This is a problem, because site-banning non-local users will still leave content
/// they've posted to our local communities, on other servers.
///
/// So when doing a site ban for a non-local user, you need to federate/send a
/// community ban for every local community they've participated in.
/// See https://github.com/LemmyNet/lemmy/issues/4118
#[tracing::instrument(skip_all)]
pub(crate) async fn ban_nonlocal_user_from_local_communities(
  local_user_view: &LocalUserView,
  target: &Person,
  ban: bool,
  reason: &Option<String>,
  remove_data: &Option<bool>,
  expires: &Option<i64>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let ids = Person::list_local_community_ids(&mut context.pool(), target.id).await?;

  for community_id in ids {
    let expires_dt = check_expire_time(*expires)?;

    // Ban / unban them from our local communities
    let community_user_ban_form = CommunityPersonBanForm {
      community_id,
      person_id: target.id,
      expires: Some(expires_dt),
    };

    if ban {
      // Ignore all errors for these
      CommunityPersonBan::ban(&mut context.pool(), &community_user_ban_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)
        .ok();

      // Also unsubscribe them from the community, if they are subscribed
      let community_follower_form = CommunityFollowerForm {
        community_id,
        person_id: target.id,
        pending: false,
      };

      CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
        .await
        .ok();
    } else {
      CommunityPersonBan::unban(&mut context.pool(), &community_user_ban_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)
        .ok();
    }

    // Mod tables
    let form = ModBanFromCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: target.id,
      community_id,
      reason: reason.clone(),
      banned: Some(ban),
      expires: expires_dt,
    };

    ModBanFromCommunity::create(&mut context.pool(), &form).await?;

    // Federate the ban from community
    let ban_from_community = BanFromCommunity {
      community_id,
      person_id: target.id,
      ban,
      reason: reason.clone(),
      remove_data: *remove_data,
      expires: *expires,
    };

    ActivityChannel::submit_activity(
      SendActivityData::BanFromCommunity {
        moderator: local_user_view.person.clone(),
        community_id,
        target: target.clone(),
        data: ban_from_community,
      },
      context,
    )
    .await?;
  }

  Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn local_user_view_from_jwt(
  jwt: &str,
  context: &LemmyContext,
) -> Result<LocalUserView, LemmyError> {
  let local_user_id = Claims::validate(jwt, context)
    .await
    .with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_user_valid(&local_user_view.person)?;

  Ok(local_user_view)
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;

  #[test]
  fn test_build_totp() {
    let generated_secret = generate_totp_2fa_secret();
    let totp = build_totp_2fa("lemmy", "my_name", &generated_secret);
    assert!(totp.is_ok());
  }
}
