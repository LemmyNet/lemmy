use crate::{settings::structs::Settings, LemmyError};
use anyhow::anyhow;
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;
use url::Url;
use webpage::HTML;

#[derive(Clone, Debug, Error)]
#[error("Error sending request, {0}")]
struct SendError(pub String);

#[derive(Clone, Debug, Error)]
#[error("Error receiving response, {0}")]
pub struct RecvError(pub String);

pub async fn retry<F, Fut, T>(f: F) -> Result<T, reqwest::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<T, reqwest::Error>>,
{
  retry_custom(|| async { Ok((f)().await) }).await
}

async fn retry_custom<F, Fut, T>(f: F) -> Result<T, reqwest::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<Result<T, reqwest::Error>, reqwest::Error>>,
{
  let mut response: Option<Result<T, reqwest::Error>> = None;

  for _ in 0u8..3 {
    match (f)().await? {
      Ok(t) => return Ok(t),
      Err(e) => {
        if e.is_timeout() {
          response = Some(Err(e));
          continue;
        }
        return Err(e);
      }
    }
  }

  response.expect("retry http request")
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PostLinkTags {
  pub title: Option<String>,
  pub description: Option<String>,
  thumbnail_url: Option<Url>,
  pub html: Option<String>,
}

/// Fetches the post link html tags (like title, description, thumbnail, etc)
pub async fn fetch_post_link_tags(client: &Client, url: &Url) -> Result<PostLinkTags, LemmyError> {
  let response = retry(|| client.get(url.as_str()).send()).await?;

  let html = response
    .text()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  let tags = html_to_post_link_tags(&html)?;

  Ok(tags)
}

fn html_to_post_link_tags(html: &str) -> Result<PostLinkTags, LemmyError> {
  let page = HTML::from_string(html.to_string(), None)?;

  let page_title = page.title;
  let page_description = page.description;

  let og_description = page
    .opengraph
    .properties
    .get("description")
    .map(|t| t.to_string());
  let og_title = page
    .opengraph
    .properties
    .get("title")
    .map(|t| t.to_string());
  let og_image = page
    .opengraph
    .images
    .get(0)
    .map(|ogo| Url::parse(&ogo.url).ok())
    .flatten();

  let title = og_title.or(page_title);
  let description = og_description.or(page_description);
  let thumbnail_url = og_image;

  Ok(PostLinkTags {
    title,
    description,
    thumbnail_url,
    html: None,
  })
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsFile {
  file: String,
  delete_token: String,
}

pub(crate) async fn fetch_pictrs(
  client: &Client,
  image_url: &Url,
) -> Result<Option<PictrsResponse>, LemmyError> {
  if let Some(pictrs_url) = Settings::get().pictrs_url {
    is_image_content_type(client, image_url).await?;

    let fetch_url = format!(
      "{}/image/download?url={}",
      pictrs_url,
      utf8_percent_encode(image_url.as_str(), NON_ALPHANUMERIC) // TODO this might not be needed
    );

    let response = retry(|| client.get(&fetch_url).send()).await?;

    let response: PictrsResponse = response
      .json()
      .await
      .map_err(|e| RecvError(e.to_string()))?;

    if response.msg == "ok" {
      Ok(Some(response))
    } else {
      Err(anyhow!("{}", &response.msg).into())
    }
  } else {
    Ok(None)
  }
}

/// Both are options, since the URL might be either an html page, or an image
pub async fn fetch_post_links_and_pictrs_data(
  client: &Client,
  url: Option<&Url>,
) -> Result<(Option<PostLinkTags>, Option<Url>), LemmyError> {
  match &url {
    Some(url) => {
      // Fetch post-links data
      let post_links_res_option = fetch_post_link_tags(client, url).await.ok();

      // Fetch pictrs thumbnail
      let pictrs_hash = match &post_links_res_option {
        Some(post_link_res) => match &post_link_res.thumbnail_url {
          Some(post_links_thumbnail_url) => fetch_pictrs(client, post_links_thumbnail_url)
            .await?
            .map(|r| r.files[0].file.to_owned()),
          // Try to generate a small thumbnail if there's a full sized one from post-links
          None => fetch_pictrs(client, url)
            .await?
            .map(|r| r.files[0].file.to_owned()),
        },
        None => fetch_pictrs(client, url)
          .await?
          .map(|r| r.files[0].file.to_owned()),
      };

      // The full urls are necessary for federation
      let pictrs_thumbnail = pictrs_hash
        .map(|p| {
          Url::parse(&format!(
            "{}/pictrs/image/{}",
            Settings::get().get_protocol_and_hostname(),
            p
          ))
          .ok()
        })
        .flatten();

      Ok((post_links_res_option, pictrs_thumbnail))
    }
    None => Ok((None, None)),
  }
}

async fn is_image_content_type(client: &Client, test: &Url) -> Result<(), LemmyError> {
  let response = retry(|| client.get(test.to_owned()).send()).await?;
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

#[cfg(test)]
mod tests {
  use crate::request::fetch_post_link_tags;
  use url::Url;

  use super::PostLinkTags;

  // These helped with testing
  #[actix_rt::test]
  async fn test_post_links() {
    let client = reqwest::Client::default();
    let sample_url = Url::parse("https://www.redspark.nu/en/peoples-war/district-leader-of-chand-led-cpn-arrested-in-bhojpur/").unwrap();
    let sample_res = fetch_post_link_tags(&client, &sample_url).await.unwrap();
    assert_eq!(
      PostLinkTags {
        title: Some("District Leader Of Chand Led CPN Arrested In Bhojpur - Redspark".to_string()),
        description: Some("BHOJPUR: A district leader of the outlawed Netra Bikram Chand alias Biplav-led outfit has been arrested. According to District Police".to_string()),
        thumbnail_url: Some(Url::parse("https://www.redspark.nu/wp-content/uploads/2020/03/netra-bikram-chand-attends-program-1272019033653-1000x0-845x653-1.jpg").unwrap()),
        html: None,
      }, sample_res);

    let youtube_url = Url::parse("https://www.youtube.com/watch?v=IquO_TcMZIQ").unwrap();
    let youtube_res = fetch_post_link_tags(&client, &youtube_url).await.unwrap();
    assert_eq!(
      PostLinkTags {
        title: Some("A Hard Look at Rent and Rent Seeking with Michael Hudson & Pepe Escobar".to_string()),
        description: Some("An interactive discussion on wealth inequality and the “Great Game” on the control of natural resources.In this webinar organized jointly by the Henry George...".to_string()),
        thumbnail_url: Some(Url::parse("https://i.ytimg.com/vi/IquO_TcMZIQ/maxresdefault.jpg").unwrap()),
        html: None,
      }, youtube_res);
  }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }
}
