use actix_web::{
  http::header::{self, CacheDirective},
  web::Data,
  HttpResponse,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{newtypes::DbUrl, source::post::Post};
use lemmy_utils::error::LemmyResult;
use sitemap_rs::{url::Url, url_set::UrlSet};
use tracing::info;

async fn generate_urlset(
  posts: Vec<(DbUrl, chrono::DateTime<chrono::Utc>)>,
) -> LemmyResult<UrlSet> {
  let urls = posts
    .into_iter()
    .map_while(|post| {
      Url::builder(post.0.to_string())
        .last_modified(post.1.into())
        .build()
        .ok()
    })
    .collect();

  Ok(UrlSet::new(urls)?)
}

pub async fn get_sitemap(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  info!("Generating sitemap...",);
  let posts = Post::list_for_sitemap(&mut context.pool()).await?;
  info!("Loaded latest {} posts", posts.len());

  let mut buf = Vec::<u8>::new();
  generate_urlset(posts).await?.write(&mut buf)?;

  Ok(
    HttpResponse::Ok()
      .content_type("application/xml")
      .insert_header(header::CacheControl(vec![CacheDirective::MaxAge(3_600)])) // 1 h
      .body(buf),
  )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {

  use crate::sitemap::generate_urlset;
  use chrono::{DateTime, NaiveDate, Utc};
  use elementtree::Element;
  use lemmy_db_schema::newtypes::DbUrl;
  use pretty_assertions::assert_eq;
  use url::Url;

  #[tokio::test]
  async fn test_generate_urlset() {
    let posts: Vec<(DbUrl, DateTime<Utc>)> = vec![
      (
        Url::parse("https://example.com").unwrap().into(),
        NaiveDate::from_ymd_opt(2022, 12, 1)
          .unwrap()
          .and_hms_opt(9, 10, 11)
          .unwrap()
          .and_utc(),
      ),
      (
        Url::parse("https://lemmy.ml").unwrap().into(),
        NaiveDate::from_ymd_opt(2023, 1, 1)
          .unwrap()
          .and_hms_opt(1, 2, 3)
          .unwrap()
          .and_utc(),
      ),
    ];

    let mut buf = Vec::<u8>::new();
    generate_urlset(posts)
      .await
      .unwrap()
      .write(&mut buf)
      .unwrap();
    let root = Element::from_reader(buf.as_slice()).unwrap();

    assert_eq!(root.tag().name(), "urlset");
    assert_eq!(root.child_count(), 2);

    assert!(root.children().all(|url| url.tag().name() == "url"));
    assert!(root.children().all(|url| url.child_count() == 2));
    assert!(root.children().all(|url| url
      .children()
      .next()
      .is_some_and(|element| element.tag().name() == "loc")));
    assert!(root.children().all(|url| url
      .children()
      .nth(1)
      .is_some_and(|element| element.tag().name() == "lastmod")));

    assert_eq!(
      root
        .children()
        .next()
        .unwrap()
        .children()
        .find(|element| element.tag().name() == "loc")
        .unwrap()
        .text(),
      "https://example.com/"
    );
    assert_eq!(
      root
        .children()
        .next()
        .unwrap()
        .children()
        .find(|element| element.tag().name() == "lastmod")
        .unwrap()
        .text(),
      "2022-12-01T09:10:11+00:00"
    );
    assert_eq!(
      root
        .children()
        .nth(1)
        .unwrap()
        .children()
        .find(|element| element.tag().name() == "loc")
        .unwrap()
        .text(),
      "https://lemmy.ml/"
    );
    assert_eq!(
      root
        .children()
        .nth(1)
        .unwrap()
        .children()
        .find(|element| element.tag().name() == "lastmod")
        .unwrap()
        .text(),
      "2023-01-01T01:02:03+00:00"
    );
  }
}
