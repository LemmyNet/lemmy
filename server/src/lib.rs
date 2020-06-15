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
pub extern crate rand;
pub extern crate regex;
pub extern crate rss;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate sha2;
pub extern crate strum;

pub mod api;
pub mod apub;
pub mod db;
pub mod rate_limit;
pub mod routes;
pub mod schema;
pub mod settings;
pub mod version;
pub mod websocket;

use actix_web::dev::ConnectionInfo;
use chrono::{DateTime, NaiveDateTime, Utc};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, SmtpClient, Transport};
use lettre_email::Email;
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::{Regex, RegexBuilder};
use serde::Deserialize;

use crate::settings::Settings;

pub type ConnectionId = usize;
pub type PostId = i32;
pub type CommunityId = i32;
pub type UserId = i32;
pub type IPAddr = String;

pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp(time, 0)
}

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub fn is_image_content_type(test: &str) -> Result<(), failure::Error> {
  if attohttpc::get(test)
    .send()?
    .headers()
    .get("Content-Type")
    .ok_or_else(|| format_err!("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(format_err!("Not an image type."))
  }
}

pub fn remove_slurs(test: &str) -> String {
  POST_FILTER_REGEX.replace_all(test, "*removed*").to_string()
}

pub fn slur_check(test: &str) -> Result<(), Vec<&str>> {
  let mut matches: Vec<&str> = POST_FILTER_REGEX.find_iter(test).map(|mat| mat.as_str()).collect();

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

pub fn extract_usernames(test: &str) -> Vec<&str> {
  let mut matches: Vec<&str> = USERNAME_MATCHES_REGEX
    .find_iter(test)
    .map(|mat| mat.as_str())
    .collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  // Remove /u/
  matches.iter().map(|t| &t[3..]).collect()
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

pub fn fetch_iframely(url: &str) -> Result<IframelyResponse, failure::Error> {
  let fetch_url = format!("http://iframely/oembed?url={}", url);
  let text: String = attohttpc::get(&fetch_url).send()?.text()?;
  let res: IframelyResponse = serde_json::from_str(&text)?;
  Ok(res)
}

#[derive(Deserialize, Debug)]
pub struct PictshareResponse {
  status: String,
  url: String,
}

pub fn fetch_pictshare(image_url: &str) -> Result<PictshareResponse, failure::Error> {
  is_image_content_type(image_url)?;

  let fetch_url = format!(
    "http://pictshare/api/geturl.php?url={}",
    utf8_percent_encode(image_url, NON_ALPHANUMERIC)
  );
  let text = attohttpc::get(&fetch_url).send()?.text()?;
  let res: PictshareResponse = serde_json::from_str(&text)?;
  Ok(res)
}

fn fetch_iframely_and_pictshare_data(
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
        match fetch_iframely(url) {
          Ok(res) => (res.title, res.description, res.thumbnail_url, res.html),
          Err(e) => {
            error!("iframely err: {}", e);
            (None, None, None, None)
          }
        };

      // Fetch pictshare thumbnail
      let pictshare_thumbnail = match iframely_thumbnail_url {
        Some(iframely_thumbnail_url) => match fetch_pictshare(&iframely_thumbnail_url) {
          Ok(res) => Some(res.url),
          Err(e) => {
            error!("pictshare err: {}", e);
            None
          }
        },
        // Try to generate a small thumbnail if iframely is not supported
        None => match fetch_pictshare(&url) {
          Ok(res) => Some(res.url),
          Err(e) => {
            error!("pictshare err: {}", e);
            None
          }
        },
      };

      (
        iframely_title,
        iframely_description,
        iframely_html,
        pictshare_thumbnail,
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
    .remote()
    .unwrap_or("127.0.0.1:12345")
    .split(':')
    .next()
    .unwrap_or("127.0.0.1")
    .to_string()
}

pub fn is_valid_username(name: &str) -> bool {
  VALID_USERNAME_REGEX.is_match(name)
}

#[cfg(test)]
mod tests {
  use crate::{
    extract_usernames, is_email_regex, is_image_content_type, is_valid_username, remove_slurs,
    slur_check, slurs_vec_to_str,
  };

  #[test]
  fn test_image() {
    assert!(is_image_content_type("https://1734811051.rsc.cdn77.org/data/images/full/365645/as-virus-kills-navajos-in-their-homes-tribal-women-provide-lifeline.jpg?w=600?w=650").is_ok());
    assert!(is_image_content_type(
      "https://twitter.com/BenjaminNorton/status/1259922424272957440?s=20"
    )
    .is_err());
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
  fn test_slur_filter() {
    let test =
      "I was minding my own business walking past the HOA board when a bunch of hiPpIes jumped out from behind a bush and accosted me for being a banker. I corrected them and told them I do not work for a bank, but in fact I am a meat popsicle. After this we went on our merry way.";
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "I was minding my own business walking past the *removed* when a bunch of *removed* jumped out from behind a bush and accosted me for being a *removed*. I corrected them and told them I do not work for a *removed*, but in fact I am a *removed*. After this we went on our merry way."
        .to_string()
    );

    let has_slurs_vec = vec![
      "HOA board",
      "bank",
      "banker",
      "hiPpIes",
      "meat popsicle",
    ];
    let has_slurs_err_str = "No slurs - HOA board, bank, banker, hiPpIes, meat popsicle";

    assert_eq!(slur_check(test), Err(has_slurs_vec));
    assert_eq!(slur_check(slur_free), Ok(()));
    if let Err(slur_vec) = slur_check(test) {
      assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
    }
  }

  #[test]
  fn test_extract_usernames() {
    let usernames = extract_usernames("this is a user mention for [/u/testme](/u/testme) and thats all. Oh [/u/another](/u/another) user. And the first again [/u/testme](/u/testme) okay");
    let expected = vec!["another", "testme"];
    assert_eq!(usernames, expected);
  }

  // These helped with testing
  // #[test]
  // fn test_iframely() {
  //   let res = fetch_iframely("https://www.redspark.nu/?p=15341");
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
  static ref POST_FILTER_REGEX: Regex = RegexBuilder::new(&std::env::var("LEMMY_POST_FILTER_REGEX").unwrap_or(if cfg!(test) { "(hippies?|bank(ers?)?|hoa board|meat popsic(al|le)s?)" } else { ".^" }.into())).case_insensitive(true).build().unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
  static ref VALID_USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
}
