use crate::newtypes::{ActivityId, CommunityId, DbUrl};
use chrono::{DateTime, Utc};
use diesel::Queryable;
use lemmy_db_schema_file::{
  enums::ActorType,
  schema::{received_activity, sent_activity},
};
use serde_json::Value;
use std::{collections::HashSet, fmt::Debug};
use url::Url;

#[derive(FromSqlRow, PartialEq, Eq, Debug, Default, Clone)]
/// describes where an activity should be sent
pub struct ActivitySendTargets {
  /// send to these inboxes explicitly
  pub inboxes: HashSet<Url>,
  /// send to all followers of these local communities
  pub community_followers_of: Option<CommunityId>,
  /// send to all remote instances
  pub all_instances: bool,
}

// todo: in different file?
impl ActivitySendTargets {
  pub fn empty() -> ActivitySendTargets {
    ActivitySendTargets::default()
  }
  pub fn to_inbox(url: Url) -> ActivitySendTargets {
    let mut a = ActivitySendTargets::empty();
    a.inboxes.insert(url);
    a
  }
  pub fn to_local_community_followers(id: CommunityId) -> ActivitySendTargets {
    let mut a = ActivitySendTargets::empty();
    a.community_followers_of = Some(id);
    a
  }
  pub fn to_all_instances() -> ActivitySendTargets {
    let mut a = ActivitySendTargets::empty();
    a.all_instances = true;
    a
  }
  pub fn set_all_instances(&mut self) {
    self.all_instances = true;
  }

  pub fn add_inbox(&mut self, inbox: Url) {
    self.inboxes.insert(inbox);
  }
  pub fn add_inboxes(&mut self, inboxes: Vec<DbUrl>) {
    self.inboxes.extend(inboxes.into_iter().map(Into::into));
  }
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(table_name = sent_activity))]
pub struct SentActivity {
  pub id: ActivityId,
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
  pub published_at: DateTime<Utc>,
  pub send_inboxes: Vec<Option<DbUrl>>,
  pub send_community_followers_of: Option<CommunityId>,
  pub send_all_instances: bool,
  pub actor_type: ActorType,
  pub actor_apub_id: Option<DbUrl>,
}

#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = sent_activity))]
pub struct SentActivityForm {
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
  pub send_inboxes: Vec<Option<DbUrl>>,
  pub send_community_followers_of: Option<i32>,
  pub send_all_instances: bool,
  pub actor_type: ActorType,
  pub actor_apub_id: DbUrl,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(primary_key(ap_id)))]
#[cfg_attr(feature = "full", diesel(table_name = received_activity))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct ReceivedActivity {
  pub ap_id: DbUrl,
  pub published_at: DateTime<Utc>,
}
