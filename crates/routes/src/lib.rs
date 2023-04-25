use actix_web::{
  error::ParseError,
  http::header::{Header, HeaderName, HeaderValue, TryIntoHeaderValue},
  HttpMessage,
};
use lemmy_api_common::sensitive::Sensitive;
use serde::Deserialize;
use std::convert::Infallible;

pub mod feeds;
pub mod images;
pub mod nodeinfo;
pub mod webfinger;

#[derive(Clone)]
pub struct AuthHeader(pub Option<Sensitive<String>>);

impl Header for AuthHeader {
  fn name() -> HeaderName {
    HeaderName::from_static("auth")
  }

  fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
    Ok(AuthHeader(
      msg
        .headers()
        .get(Self::name())
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| Sensitive::new(v.to_string())),
    ))
  }
}

impl TryIntoHeaderValue for AuthHeader {
  type Error = Infallible;

  fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
    unimplemented!()
  }
}

#[derive(Deserialize)]
pub struct WithAuth<T> {
  #[serde(flatten)]
  pub data: T,
  pub auth: Option<Sensitive<String>>,
}
