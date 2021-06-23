use crate::activities_new::follow::Accept;
use activitystreams::{
  base::AnyBase,
  error::DomainError,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use std::marker::PhantomData;
use url::Url;

// for now, limit it to activity routing only, no http sigs, parsing or any of that
// need to route in this order:
// 1. recipient actor
// 2. activity type
// 3. inner object (recursively until object is empty or an url)

// library part
// todo: move this to separate crate

// TODO: turn this into a trait in which app has to implement the following functions:
// .checkIdValid() - for unique, instance block etc
// .checkHttpSig::<RequestType>()
// .fetchObject() - for custom http client
// .checkActivity() - for common validity checks
struct InboxConfig {
  //actors: Vec<ActorConfig>,
}

impl InboxConfig {
  fn shared_inbox_handler() {
    todo!()
  }
}

pub fn verify_domains_match(a: &Url, b: &Url) -> Result<(), LemmyError> {
  if a.domain() != b.domain() {
    return Err(DomainError.into());
  }
  Ok(())
}

// todo: later add a similar trait SendActivity
// todo: maybe add a separate method verify()
#[async_trait::async_trait(?Send)]
pub trait ReceiveActivity {
  // todo: later handle request_counter completely inside library
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError>;
}

// todo: instead of phantomdata, might use option<kind> to cache the fetched object (or just fetch on construction)
pub struct ObjectId<'a, Kind>(Url, &'a PhantomData<Kind>);

impl<Kind> ObjectId<'_, Kind> {
  pub fn url(self) -> Url {
    self.0
  }
  pub fn dereference(self) -> Result<Kind, LemmyError> {
    // todo: fetch object from http or database
    todo!()
  }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity<Kind> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  id: Url,

  /// type-specific fields
  #[serde(flatten)]
  pub inner: Kind,

  // unparsed fields
  // todo: can probably remove this field
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl<Kind> Activity<Kind> {
  pub fn id_unchecked(&self) -> &Url {
    &self.id
  }
}

// application part

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum PersonAcceptedActivitiesNew {
  Accept(Accept),
}

// todo: there should be a better way to do this (maybe needs a derive macro)
#[async_trait::async_trait(?Send)]
impl ReceiveActivity for PersonAcceptedActivitiesNew {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    use PersonAcceptedActivitiesNew::*;
    self.receive(context, request_counter).await
  }
}
