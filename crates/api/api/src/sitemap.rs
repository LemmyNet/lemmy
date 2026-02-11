use actix_web::{
  HttpResponse,
  http::header::{self, CacheDirective},
  web::Data,
};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::source::post::Post;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::dburl::DbUrl;
use lemmy_utils::error::LemmyResult;
use sitemap_rs::{url::Url, url_set::UrlSet};
use tracing::info;

fn generate_urlset(posts: Vec<(DbUrl, chrono::DateTime<chrono::Utc>)>) -> LemmyResult<UrlSet> {
  let urls = posts
    .into_iter()
    .map_while(|(url, date_time)| {
      Url::builder(url.to_string())
        .last_modified(date_time.into())
        .build()
        .ok()
    })
    .collect();

  Ok(UrlSet::new(urls)?)
}

pub async fn get_sitemap(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  check_private_instance(&None, &local_site)?;

  info!("Generating sitemap...",);
  let posts = Post::list_for_sitemap(&mut context.pool()).await?;
  info!("Loaded latest {} posts", posts.len());

  let mut buf = Vec::<u8>::new();
  generate_urlset(posts)?.write(&mut buf)?;

  Ok(
    HttpResponse::Ok()
      .content_type("application/xml")
      .insert_header(header::CacheControl(vec![CacheDirective::MaxAge(3_600)])) // 1 h
      .body(buf),
  )
}

#[cfg(test)]
pub(crate) mod tests {

  use crate::sitemap::generate_urlset;
  use chrono::{DateTime, NaiveDate, Utc};
  use elementtree::Element;
  use lemmy_diesel_utils::dburl::DbUrl;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use url::Url;

  #[tokio::test]
  async fn test_generate_urlset() -> LemmyResult<()> {
    let posts: Vec<(DbUrl, DateTime<Utc>)> = vec![
      (
        Url::parse("https://example.com")?.into(),
        NaiveDate::from_ymd_opt(2022, 12, 1)
          .unwrap_or_default()
          .and_hms_opt(9, 10, 11)
          .unwrap_or_default()
          .and_utc(),
      ),
      (
        Url::parse("https://lemmy.ml")?.into(),
        NaiveDate::from_ymd_opt(2023, 1, 1)
          .unwrap_or_default()
          .and_hms_opt(1, 2, 3)
          .unwrap_or_default()
          .and_utc(),
      ),
    ];

    let mut buf = Vec::<u8>::new();
    generate_urlset(posts)?.write(&mut buf)?;
    let root = Element::from_reader(buf.as_slice())?;

    assert_eq!(root.tag().name(), "urlset");
    assert_eq!(root.child_count(), 2);

    assert!(root.children().all(|url| url.tag().name() == "url"));
    assert!(root.children().all(|url| url.child_count() == 2));
    assert!(root.children().all(|url| {
      url
        .children()
        .next()
        .is_some_and(|element| element.tag().name() == "loc")
    }));
    assert!(root.children().all(|url| {
      url
        .children()
        .nth(1)
        .is_some_and(|element| element.tag().name() == "lastmod")
    }));

    assert_eq!(
      root
        .children()
        .next()
        .and_then(|n| n.children().find(|element| element.tag().name() == "loc"))
        .map(Element::text)
        .unwrap_or_default(),
      "https://example.com/"
    );
    assert_eq!(
      root
        .children()
        .next()
        .and_then(|n| n
          .children()
          .find(|element| element.tag().name() == "lastmod"))
        .map(Element::text)
        .unwrap_or_default(),
      "2022-12-01T09:10:11+00:00"
    );
    assert_eq!(
      root
        .children()
        .nth(1)
        .and_then(|n| n.children().find(|element| element.tag().name() == "loc"))
        .map(Element::text)
        .unwrap_or_default(),
      "https://lemmy.ml/"
    );
    assert_eq!(
      root
        .children()
        .nth(1)
        .and_then(|n| n
          .children()
          .find(|element| element.tag().name() == "lastmod"))
        .map(Element::text)
        .unwrap_or_default(),
      "2023-01-01T01:02:03+00:00"
    );

    Ok(())
  }
}
