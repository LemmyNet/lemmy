#![recursion_limit = "512"]
#[macro_use]
extern crate lazy_static;
extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bcrypt;
extern crate captcha;
extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate jsonwebtoken;
extern crate log;
extern crate openssl;
extern crate reqwest;
extern crate rss;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate strum;

pub mod api;
pub mod apub;
pub mod code_migrations;
pub mod request;
pub mod routes;
pub mod version;
pub mod websocket;

use crate::{
  request::{retry, RecvError},
  websocket::chat_server::ChatServer,
};
use actix::Addr;
use anyhow::anyhow;
use background_jobs::QueueHandle;
use lemmy_db::DbPool;
use lemmy_utils::{apub::get_apub_protocol_string, settings::Settings, LemmyError};
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Deserialize;
use std::process::Command;

pub struct LemmyContext {
  pub pool: DbPool,
  pub chat_server: Addr<ChatServer>,
  pub client: Client,
  pub activity_queue: QueueHandle,
}

impl LemmyContext {
  pub fn new(
    pool: DbPool,
    chat_server: Addr<ChatServer>,
    client: Client,
    activity_queue: QueueHandle,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      chat_server,
      client,
      activity_queue,
    }
  }
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }
  pub fn chat_server(&self) -> &Addr<ChatServer> {
    &self.chat_server
  }
  pub fn client(&self) -> &Client {
    &self.client
  }
  pub fn activity_queue(&self) -> &QueueHandle {
    &self.activity_queue
  }
}

impl Clone for LemmyContext {
  fn clone(&self) -> Self {
    LemmyContext::new(
      self.pool.clone(),
      self.chat_server.clone(),
      self.client.clone(),
      self.activity_queue.clone(),
    )
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

  let response = retry(|| client.get(&fetch_url).send()).await?;

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

  let response = retry(|| client.get(&fetch_url).send()).await?;

  let response: PictrsResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  if response.msg == "ok" {
    Ok(response)
  } else {
    Err(anyhow!("{}", &response.msg).into())
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
      let pictrs_hash = match iframely_thumbnail_url {
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

      // The full urls are necessary for federation
      let pictrs_thumbnail = if let Some(pictrs_hash) = pictrs_hash {
        Some(format!(
          "{}://{}/pictrs/image/{}",
          get_apub_protocol_string(),
          Settings::get().hostname,
          pictrs_hash
        ))
      } else {
        None
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

pub async fn is_image_content_type(client: &Client, test: &str) -> Result<(), LemmyError> {
  let response = retry(|| client.get(test).send()).await?;

  if response
    .headers()
    .get("Content-Type")
    .ok_or_else(|| anyhow!("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(anyhow!("Not an image type.").into())
  }
}

pub fn captcha_espeak_wav_base64(captcha: &str) -> Result<String, LemmyError> {
  let mut built_text = String::new();

  // Building proper speech text for espeak
  for mut c in captcha.chars() {
    let new_str = if c.is_alphabetic() {
      if c.is_lowercase() {
        c.make_ascii_uppercase();
        format!("lower case {} ... ", c)
      } else {
        c.make_ascii_uppercase();
        format!("capital {} ... ", c)
      }
    } else {
      format!("{} ...", c)
    };

    built_text.push_str(&new_str);
  }

  espeak_wav_base64(&built_text)
}

pub fn espeak_wav_base64(text: &str) -> Result<String, LemmyError> {
  // Make a temp file path
  let uuid = uuid::Uuid::new_v4().to_string();
  let file_path = format!("/tmp/lemmy_espeak_{}.wav", &uuid);

  // Write the wav file
  Command::new("espeak")
    .arg("-w")
    .arg(&file_path)
    .arg(text)
    .status()?;

  // Read the wav file bytes
  let bytes = std::fs::read(&file_path)?;

  // Delete the file
  std::fs::remove_file(file_path)?;

  // Convert to base64
  let base64 = base64::encode(bytes);

  Ok(base64)
}

#[cfg(test)]
mod tests {
  use crate::{captcha_espeak_wav_base64, is_image_content_type};

  #[test]
  fn test_image() {
    actix_rt::System::new("tset_image").block_on(async move {
      let client = reqwest::Client::default();
      assert!(is_image_content_type(&client, "https://1734811051.rsc.cdn77.org/data/images/full/365645/as-virus-kills-navajos-in-their-homes-tribal-women-provide-lifeline.jpg?w=600?w=650").await.is_ok());
      assert!(is_image_content_type(&client,
                                    "https://twitter.com/BenjaminNorton/status/1259922424272957440?s=20"
      )
        .await.is_err()
      );
    });
  }

  #[test]
  fn test_espeak() {
    assert!(captcha_espeak_wav_base64("WxRt2l").is_ok())
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
}
