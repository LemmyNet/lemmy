pub mod queries;

use chrono::TimeDelta;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 20;
pub const FETCH_LIMIT_MAX: usize = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: TimeDelta = TimeDelta::days(31);
pub const RANK_DEFAULT: f32 = 0.0001;
pub const DELETED_REPLACEMENT_TEXT: &str = "*Permanently Deleted*";

pub fn limit_fetch(limit: Option<i64>, no_limit: Option<bool>) -> LemmyResult<i64> {
  Ok(if no_limit.unwrap_or_default() {
    i64::MAX
  } else {
    match limit {
      Some(limit) => limit_fetch_check(limit)?,
      None => FETCH_LIMIT_DEFAULT,
    }
  })
}

pub fn limit_fetch_check(limit: i64) -> LemmyResult<i64> {
  if !(1..=FETCH_LIMIT_MAX.try_into()?).contains(&limit) {
    Err(LemmyErrorType::InvalidFetchLimit.into())
  } else {
    Ok(limit)
  }
}

pub(crate) fn format_actor_url(
  name: &str,
  domain: &str,
  prefix: char,
  settings: &Settings,
) -> LemmyResult<Url> {
  let local_protocol_and_hostname = settings.get_protocol_and_hostname();
  let local_hostname = &settings.hostname;
  let url = if domain != local_hostname {
    format!("{local_protocol_and_hostname}/{prefix}/{name}@{domain}",)
  } else {
    format!("{local_protocol_and_hostname}/{prefix}/{name}")
  };
  Ok(Url::parse(&url)?)
}
