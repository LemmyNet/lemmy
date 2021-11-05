use crate::generate_outbox_url;
use activitystreams::collection::kind::OrderedCollectionType;
use lemmy_db_schema::source::person::Person;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersonOutbox {
  r#type: OrderedCollectionType,
  id: Url,
  ordered_items: Vec<()>,
  total_items: i32,
}

impl PersonOutbox {
  pub(crate) async fn new(user: Person) -> Result<PersonOutbox, LemmyError> {
    Ok(PersonOutbox {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_outbox_url(&user.actor_id)?.into(),
      ordered_items: vec![],
      total_items: 0,
    })
  }
}
