use actix_web::middleware::DefaultHeaders;

/// Adds a cache header to requests
///
/// Common cache amounts are:
///   * 1 hour = 60s * 60m = `3600` seconds
///   * 3 days = 60s * 60m * 24h * 3d = `259200` seconds
///
/// Mastodon & other activitypub server defaults to 3d
pub fn cache_header(seconds: usize) -> DefaultHeaders {
  DefaultHeaders::new().add(("Cache-Control", format!("public, max-age={seconds}")))
}

/// Set a 1 hour cache time
pub fn cache_1hour() -> DefaultHeaders {
  cache_header(3600)
}

/// Set a 3 day cache time
pub fn cache_3days() -> DefaultHeaders {
  cache_header(259200)
}
