use crate::Error;
use url::Url;

pub fn verify_domains_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a.domain() != b.domain() {
    return Err(Error::DomainError);
  }
  Ok(())
}

pub fn verify_urls_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a != b {
    return Err(Error::DomainError);
  }
  Ok(())
}
