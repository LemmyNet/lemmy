use crate::Error;
use url::Url;

pub fn verify_domains_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a.domain() != b.domain() {
    return Err(Error::UrlVerificationError("Domains do not match"));
  }
  Ok(())
}

pub fn verify_urls_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a != b {
    return Err(Error::UrlVerificationError("Urls do not match"));
  }
  Ok(())
}
