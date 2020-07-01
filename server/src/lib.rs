#![recursion_limit = "512"]
#[macro_use]
pub extern crate strum_macros;
#[macro_use]
pub extern crate lazy_static;
#[macro_use]
pub extern crate failure;
#[macro_use]
pub extern crate diesel;
pub extern crate actix;
pub extern crate actix_web;
pub extern crate bcrypt;
pub extern crate chrono;
pub extern crate comrak;
pub extern crate dotenv;
pub extern crate jsonwebtoken;
pub extern crate lettre;
pub extern crate lettre_email;
extern crate log;
pub extern crate openssl;
pub extern crate rand;
pub extern crate regex;
pub extern crate rss;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate sha2;
pub extern crate strum;

pub async fn blocking<F, T>(pool: &DbPool, f: F) -> Result<T, LemmyError>
where
  F: FnOnce(&diesel::PgConnection) -> T + Send + 'static,
  T: Send + 'static,
{
  let pool = pool.clone();
  let res = actix_web::web::block(move || {
    let conn = pool.get()?;
    let res = (f)(&conn);
    Ok(res) as Result<_, LemmyError>
  })
  .await?;

  Ok(res)
}

pub mod api;
pub mod apub;
pub mod db;
pub mod rate_limit;
pub mod request;
pub mod routes;
pub mod schema;
pub mod settings;
pub mod version;
pub mod websocket;

use crate::{
  request::{retry, RecvError},
  settings::Settings,
};
use actix_web::{client::Client, dev::ConnectionInfo};
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Utc};
use itertools::Itertools;
use lettre::{
  smtp::{
    authentication::{Credentials, Mechanism},
    extension::ClientId,
    ConnectionReuseParameters,
  },
  ClientSecurity,
  SmtpClient,
  Transport,
};
use lettre_email::Email;
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::{Regex, RegexBuilder};
use serde::Deserialize;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;
pub type ConnectionId = usize;
pub type PostId = i32;
pub type CommunityId = i32;
pub type UserId = i32;
pub type IPAddr = String;

#[derive(Debug)]
pub struct LemmyError {
  inner: failure::Error,
}

impl std::fmt::Display for LemmyError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

impl actix_web::error::ResponseError for LemmyError {}

impl<T> From<T> for LemmyError
where
  T: Into<failure::Error>,
{
  fn from(t: T) -> Self {
    LemmyError { inner: t.into() }
  }
}

pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp(time, 0)
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
  let now = Local::now();
  DateTime::<FixedOffset>::from_utc(datetime, *now.offset())
}

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub async fn is_image_content_type(client: &Client, test: &str) -> Result<(), LemmyError> {
  let response = retry(|| client.get(test).send()).await?;

  if response
    .headers()
    .get("Content-Type")
    .ok_or_else(|| format_err!("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(format_err!("Not an image type.").into())
  }
}

pub fn remove_slurs(test: &str) -> String {
  SLUR_REGEX.replace_all(test, "*removed*").to_string()
}

pub fn slur_check(test: &str) -> Result<(), Vec<&str>> {
  let mut matches: Vec<&str> = SLUR_REGEX.find_iter(test).map(|mat| mat.as_str()).collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  if matches.is_empty() {
    Ok(())
  } else {
    Err(matches)
  }
}

pub fn slurs_vec_to_str(slurs: Vec<&str>) -> String {
  let start = "No slurs - ";
  let combined = &slurs.join(", ");
  [start, combined].concat()
}

pub fn generate_random_string() -> String {
  thread_rng().sample_iter(&Alphanumeric).take(30).collect()
}

pub fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
) -> Result<(), String> {
  let email_config = Settings::get().email.ok_or("no_email_setup")?;

  let email = Email::builder()
    .to((to_email, to_username))
    .from(email_config.smtp_from_address.to_owned())
    .subject(subject)
    .html(html)
    .build()
    .unwrap();

  let mailer = if email_config.use_tls {
    SmtpClient::new_simple(&email_config.smtp_server).unwrap()
  } else {
    SmtpClient::new(&email_config.smtp_server, ClientSecurity::None).unwrap()
  }
  .hello_name(ClientId::Domain(Settings::get().hostname))
  .smtp_utf8(true)
  .authentication_mechanism(Mechanism::Plain)
  .connection_reuse(ConnectionReuseParameters::ReuseUnlimited);
  let mailer = if let (Some(login), Some(password)) =
    (&email_config.smtp_login, &email_config.smtp_password)
  {
    mailer.credentials(Credentials::new(login.to_owned(), password.to_owned()))
  } else {
    mailer
  };

  let mut transport = mailer.transport();
  let result = transport.send(email.into());
  transport.close();

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}

#[derive(Deserialize, Debug)]
pub struct IframelyResponse {
  title: Option<String>,
  description: Option<String>,
  thumbnail_url: Option<String>,
  html: Option<String>,
}

pub async fn fetch_iframely(client: &Client, url: &str) -> Result<IframelyResponse, LemmyError> {
  let fetch_url = format!("http://iframely/oembed?url={}", url);

  let mut response = retry(|| client.get(&fetch_url).send()).await?;

  let res: IframelyResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;
  Ok(res)
}

#[derive(Deserialize, Debug, Clone)]
pub struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PictrsFile {
  file: String,
  delete_token: String,
}

pub async fn fetch_pictrs(client: &Client, image_url: &str) -> Result<PictrsResponse, LemmyError> {
  is_image_content_type(client, image_url).await?;

  let fetch_url = format!(
    "http://pictrs:8080/image/download?url={}",
    utf8_percent_encode(image_url, NON_ALPHANUMERIC) // TODO this might not be needed
  );

  let mut response = retry(|| client.get(&fetch_url).send()).await?;

  let response: PictrsResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  if response.msg == "ok" {
    Ok(response)
  } else {
    Err(format_err!("{}", &response.msg).into())
  }
}

async fn fetch_iframely_and_pictrs_data(
  client: &Client,
  url: Option<String>,
) -> (
  Option<String>,
  Option<String>,
  Option<String>,
  Option<String>,
) {
  match &url {
    Some(url) => {
      // Fetch iframely data
      let (iframely_title, iframely_description, iframely_thumbnail_url, iframely_html) =
        match fetch_iframely(client, url).await {
          Ok(res) => (res.title, res.description, res.thumbnail_url, res.html),
          Err(e) => {
            error!("iframely err: {}", e);
            (None, None, None, None)
          }
        };

      // Fetch pictrs thumbnail
      let pictrs_thumbnail = match iframely_thumbnail_url {
        Some(iframely_thumbnail_url) => match fetch_pictrs(client, &iframely_thumbnail_url).await {
          Ok(res) => Some(res.files[0].file.to_owned()),
          Err(e) => {
            error!("pictrs err: {}", e);
            None
          }
        },
        // Try to generate a small thumbnail if iframely is not supported
        None => match fetch_pictrs(client, &url).await {
          Ok(res) => Some(res.files[0].file.to_owned()),
          Err(e) => {
            error!("pictrs err: {}", e);
            None
          }
        },
      };

      (
        iframely_title,
        iframely_description,
        iframely_html,
        pictrs_thumbnail,
      )
    }
    None => (None, None, None, None),
  }
}

pub fn markdown_to_html(text: &str) -> String {
  comrak::markdown_to_html(text, &comrak::ComrakOptions::default())
}

pub fn get_ip(conn_info: &ConnectionInfo) -> String {
  conn_info
    .remote_addr()
    .unwrap_or("127.0.0.1:12345")
    .split(':')
    .next()
    .unwrap_or("127.0.0.1")
    .to_string()
}

// TODO nothing is done with community / group webfingers yet, so just ignore those for now
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
  pub name: String,
  pub domain: String,
}

impl MentionData {
  pub fn is_local(&self) -> bool {
    Settings::get().hostname.eq(&self.domain)
  }
  pub fn full_name(&self) -> String {
    format!("@{}@{}", &self.name, &self.domain)
  }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
  let mut out: Vec<MentionData> = Vec::new();
  for caps in WEBFINGER_USER_REGEX.captures_iter(text) {
    out.push(MentionData {
      name: caps["name"].to_string(),
      domain: caps["domain"].to_string(),
    });
  }
  out.into_iter().unique().collect()
}

pub fn is_valid_username(name: &str) -> bool {
  VALID_USERNAME_REGEX.is_match(name)
}

pub fn is_valid_community_name(name: &str) -> bool {
  VALID_COMMUNITY_NAME_REGEX.is_match(name)
}

#[cfg(test)]
mod tests {
  use crate::{
    is_email_regex,
    is_image_content_type,
    is_valid_community_name,
    is_valid_username,
    remove_slurs,
    scrape_text_for_mentions,
    slur_check,
    slurs_vec_to_str,
  };

  #[test]
  fn test_mentions_regex() {
    let text = "Just read a great blog post by [@tedu@honk.teduangst.com](/u/test). And another by !test_community@fish.teduangst.com . Another [@lemmy@lemmy-alpha:8540](/u/fish)";
    let mentions = scrape_text_for_mentions(text);

    assert_eq!(mentions[0].name, "tedu".to_string());
    assert_eq!(mentions[0].domain, "honk.teduangst.com".to_string());
    assert_eq!(mentions[1].domain, "lemmy-alpha:8540".to_string());
  }

  #[test]
  fn test_image() {
    actix_rt::System::new("tset_image").block_on(async move {
        let client = actix_web::client::Client::default();
        assert!(is_image_content_type(&client, "https://1734811051.rsc.cdn77.org/data/images/full/365645/as-virus-kills-navajos-in-their-homes-tribal-women-provide-lifeline.jpg?w=600?w=650").await.is_ok());
        assert!(is_image_content_type(&client,
                "https://twitter.com/BenjaminNorton/status/1259922424272957440?s=20"
        )
            .await.is_err()
        );
      });
  }

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }

  #[test]
  fn test_valid_register_username() {
    assert!(is_valid_username("Hello_98"));
    assert!(is_valid_username("ten"));
    assert!(!is_valid_username("Hello-98"));
    assert!(!is_valid_username("a"));
    assert!(!is_valid_username(""));
  }

  #[test]
  fn test_valid_community_name() {
    assert!(is_valid_community_name("example"));
    assert!(is_valid_community_name("example_community"));
    assert!(!is_valid_community_name("Example"));
    assert!(!is_valid_community_name("Ex"));
    assert!(!is_valid_community_name(""));
  }

  #[test]
  fn test_slur_filter() {
    let test =
      "coons test dindu ladyboy tranny retardeds. Capitalized Niggerz. This is a bunch of other safe text.";
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );

    let has_slurs_vec = vec![
      "Niggerz",
      "coons",
      "dindu",
      "ladyboy",
      "retardeds",
      "tranny",
    ];
    let has_slurs_err_str = "No slurs - Niggerz, coons, dindu, ladyboy, retardeds, tranny";

    assert_eq!(slur_check(test), Err(has_slurs_vec));
    assert_eq!(slur_check(slur_free), Ok(()));
    if let Err(slur_vec) = slur_check(test) {
      assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
    }
  }

  // These helped with testing
  // #[test]
  // fn test_iframely() {
  //   let res = fetch_iframely(client, "https://www.redspark.nu/?p=15341").await;
  //   assert!(res.is_ok());
  // }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }

  // #[test]
  // fn test_send_email() {
  //  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
  //   assert!(result.is_ok());
  // }
}

lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref SLUR_REGEX: Regex = RegexBuilder::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|n(i|1)g(\b|g?(a|er)?(s|z)?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btr(a|@)nn?(y|ies?)|ladyboy(s?)|\b(b|re|r)tard(ed)?s?)").case_insensitive(true).build().unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
  // TODO keep this old one, it didn't work with port well tho
  // static ref WEBFINGER_USER_REGEX: Regex = Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)").unwrap();
  static ref WEBFINGER_USER_REGEX: Regex = Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._:-]+)").unwrap();
  static ref VALID_USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
  static ref VALID_COMMUNITY_NAME_REGEX: Regex = Regex::new(r"^[a-z0-9_]{3,20}$").unwrap();
}
