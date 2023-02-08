use crate::{error::LemmyError, location_info, IpAddr};
use actix_web::dev::ConnectionInfo;
use anyhow::Context;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::{Regex, RegexBuilder};
use url::Url;

static MENTIONS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._:-]+)").expect("compile regex")
});
static VALID_ACTOR_NAME_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,}$").expect("compile regex"));
static VALID_POST_TITLE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r".*\S{3,}.*").expect("compile regex"));
static VALID_MATRIX_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^@[A-Za-z0-9._=-]+:[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").expect("compile regex")
});
// taken from https://en.wikipedia.org/wiki/UTM_parameters
static CLEAN_URL_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^utm_source|utm_medium|utm_campaign|utm_term|utm_content|gclid|gclsrc|dclid|fbclid$")
    .expect("compile regex")
});

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp_opt(time, 0).expect("convert datetime")
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
  DateTime::<FixedOffset>::from_utc(
    datetime,
    FixedOffset::east_opt(0).expect("create fixed offset"),
  )
}

pub fn remove_slurs(test: &str, slur_regex: &Option<Regex>) -> String {
  return test.to_string();
}

pub(crate) fn slur_check<'a>(
  test: &'a str,
  slur_regex: &'a Option<Regex>,
) -> Result<(), Vec<&'a str>> {
  Ok(())
}

pub fn build_slur_regex(regex_str: Option<&str>) -> Option<Regex> {
  regex_str.map(|slurs| {
    RegexBuilder::new(slurs)
      .case_insensitive(true)
      .build()
      .expect("compile regex")
  })
}

pub fn check_slurs(text: &str, slur_regex: &Option<Regex>) -> Result<(), LemmyError> {
  Ok(())
}

pub fn check_slurs_opt(
  text: &Option<String>,
  slur_regex: &Option<Regex>,
) -> Result<(), LemmyError> {
  match text {
    Some(t) => check_slurs(t, slur_regex),
    None => Ok(()),
  }
}

pub(crate) fn slurs_vec_to_str(slurs: &[&str]) -> String {
  let start = "No slurs - ";
  let combined = &slurs.join(", ");
  [start, combined].concat()
}

pub fn generate_random_string() -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .map(char::from)
    .take(30)
    .collect()
}

pub fn markdown_to_html(text: &str) -> String {
  comrak::markdown_to_html(text, &comrak::ComrakOptions::default())
}

// TODO nothing is done with community / group webfingers yet, so just ignore those for now
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
  pub name: String,
  pub domain: String,
}

impl MentionData {
  pub fn is_local(&self, hostname: &str) -> bool {
    hostname.eq(&self.domain)
  }
  pub fn full_name(&self) -> String {
    format!("@{}@{}", &self.name, &self.domain)
  }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
  let mut out: Vec<MentionData> = Vec::new();
  for caps in MENTIONS_REGEX.captures_iter(text) {
    out.push(MentionData {
      name: caps["name"].to_string(),
      domain: caps["domain"].to_string(),
    });
  }
  out.into_iter().unique().collect()
}

fn has_newline(name: &str) -> bool {
  name.contains('\n')
}

pub fn is_valid_actor_name(name: &str, actor_name_max_length: usize) -> bool {
  name.chars().count() <= actor_name_max_length
    && VALID_ACTOR_NAME_REGEX.is_match(name)
    && !has_newline(name)
}

// Can't do a regex here, reverse lookarounds not supported
pub fn is_valid_display_name(name: &str, actor_name_max_length: usize) -> bool {
  !name.starts_with('@')
    && !name.starts_with('\u{200b}')
    && name.chars().count() >= 3
    && name.chars().count() <= actor_name_max_length
    && !has_newline(name)
}

pub fn is_valid_matrix_id(matrix_id: &str) -> bool {
  VALID_MATRIX_ID_REGEX.is_match(matrix_id) && !has_newline(matrix_id)
}

pub fn is_valid_post_title(title: &str) -> bool {
  VALID_POST_TITLE_REGEX.is_match(title) && !has_newline(title)
}

pub fn get_ip(conn_info: &ConnectionInfo) -> IpAddr {
  IpAddr(
    conn_info
      .realip_remote_addr()
      .unwrap_or("127.0.0.1:12345")
      .split(':')
      .next()
      .unwrap_or("127.0.0.1")
      .to_string(),
  )
}

pub fn clean_url_params(url: &Url) -> Url {
  let mut url_out = url.clone();
  if url.query().is_some() {
    let new_query = url
      .query_pairs()
      .filter(|q| !CLEAN_URL_PARAMS_REGEX.is_match(&q.0))
      .map(|q| format!("{}={}", q.0, q.1))
      .join("&");
    url_out.set_query(Some(&new_query));
  }
  url_out
}

pub fn generate_domain_url(actor_id: &Url) -> Result<String, LemmyError> {
  Ok(actor_id.host_str().context(location_info!())?.to_string())
}

#[cfg(test)]
mod tests {
  use crate::utils::{clean_url_params, is_valid_post_title};
  use url::Url;

  #[test]
  fn test_clean_url_params() {
    let url = Url::parse("https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&username=randomuser&id=123").unwrap();
    let cleaned = clean_url_params(&url);
    let expected = Url::parse("https://example.com/path/123?username=randomuser&id=123").unwrap();
    assert_eq!(expected.to_string(), cleaned.to_string());

    let url = Url::parse("https://example.com/path/123").unwrap();
    let cleaned = clean_url_params(&url);
    assert_eq!(url.to_string(), cleaned.to_string());
  }

  #[test]
  fn regex_checks() {
    assert!(!is_valid_post_title("hi"));
    assert!(is_valid_post_title("him"));
    assert!(!is_valid_post_title("n\n\n\n\nanother"));
    assert!(!is_valid_post_title("hello there!\n this is a test."));
    assert!(is_valid_post_title("hello there! this is a test."));
  }
}
