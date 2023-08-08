use actix_web::{
  http::header::{self, CacheDirective},
  web::Data,
  HttpResponse,
};
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::post::Post, utils::DbPool};
use lemmy_utils::error::LemmyResult;
use sitemap_rs::{url::Url, url_set::UrlSet};
use tracing::info;

async fn generate_urlset(pool: &mut DbPool<'_>, limit: i64) -> LemmyResult<UrlSet> {
  info!("Generating sitemap with latest {} posts...", limit);

  let posts = Post::list_for_sitemap(pool, limit).await?;

  info!("Loaded latest {} posts", posts.len());

  let mut urls = vec![];
  for post in posts {
    let entry = Url::builder(post.0.to_string())
      .last_modified(DateTime::from_utc(
        post.1,
        FixedOffset::east_opt(0).expect("Error setting timezone offset"), // TODO what is the proper timezone offset here?
      ))
      .build()
      .ok();

    if let Some(entry) = entry {
      urls.push(entry);
    }
  }

  Ok(UrlSet::new(urls)?)
}

pub async fn get_sitemap(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let mut buf = Vec::<u8>::new();
  generate_urlset(&mut context.pool(), 50_000) // max number of entries for sitemap.xml
    .await?
    .write(&mut buf)?;

  Ok(
    HttpResponse::Ok()
      .content_type("application/xml")
      .insert_header(header::CacheControl(vec![CacheDirective::MaxAge(86_400)])) // 24 h
      .body(buf),
  )
}
