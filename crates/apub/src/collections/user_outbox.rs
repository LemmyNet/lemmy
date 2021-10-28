use crate::generate_outbox_url;
use activitystreams::collection::kind::OrderedCollectionType;
use lemmy_db_schema::source::person::Person;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserOutbox {
  r#type: OrderedCollectionType,
  id: Url,
  ordered_items: Vec<()>,
  total_items: i32,
}

impl UserOutbox {
  pub(crate) async fn new(user: Person) -> Result<UserOutbox, LemmyError> {
    Ok(UserOutbox {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_outbox_url(&user.actor_id)?.into_inner(),
      ordered_items: vec![],
      total_items: 0,
    })
  }
}
