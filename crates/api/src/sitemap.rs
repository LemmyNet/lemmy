use actix_web::{
  http::header::{self, CacheDirective},
  web::Data,
  HttpResponse,
  ResponseError,
};
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::post::Post, utils::DbPool};
use lemmy_utils::error::LemmyResult;
use sitemap_rs::{url::Url, url_set::UrlSet};
use std::io::Write;
use tracing::{error, info};

async fn generate_urlset<W: Write>(
  pool: &mut DbPool<'_>,
  writer: &mut W,
  limit: i64,
) -> LemmyResult<()> {
  info!("Generating sitemap...");

  let posts = Post::list_for_sitemap(pool, limit).await?;

  info!("Loaded latest {} posts", posts.len());

  let mut urls = vec![];
  for post in posts {
    if let Some(url) = post.url {
      let entry = match Url::builder(url.to_string())
        .last_modified(DateTime::from_utc(
          post.published,
          FixedOffset::east_opt(0).expect("Error setting timezone offset"), // TODO what is the proper timezone offset here?
        ))
        .priority(0.8) // TODO what is the correct priority?
        .change_frequency(sitemap_rs::url::ChangeFrequency::Always) // TODO what is the correct change frequency?
        .build()
      {
        Ok(url_builder) => url_builder,
        Err(_) => continue,
      };

      urls.push(entry);
    }
  }

  let url_set = UrlSet::new(urls)?;

  match url_set.write(writer) {
    Ok(_) => {
      info!("Successfully generated sitemap.xml");
      Ok(())
    }
    Err(err) => Err(err.into()),
  }
}

pub async fn get_sitemap(context: Data<LemmyContext>) -> HttpResponse {
  let mut buf = Vec::<u8>::new();
  match generate_urlset(&mut context.pool(), &mut buf, 1000).await {
    Ok(_) => HttpResponse::Ok()
      .content_type("application/xml")
      .insert_header(header::CacheControl(vec![CacheDirective::MaxAge(3600u32)]))
      .body(buf),
    Err(err) => {
      error!("{}", err);
      err.error_response()
    }
  }
}
