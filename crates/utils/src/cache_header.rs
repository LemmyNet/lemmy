use actix_web::middleware::DefaultHeaders;

/// Adds a cache header to requests
/// use 3 days (60s * 60m * 24h * 3d = 259200 seconds) as the cache duration
/// Mastodon & other activitypub server defaults to 3d
pub fn cache_header() -> DefaultHeaders {
  DefaultHeaders::new().add(("Cache-Control", "public, max-age=259200"))
}
