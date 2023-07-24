use crate::{
  newtypes::{CommunityId, DbUrl},
  schema::sent_activity,
};
use diesel::{
  deserialize::FromSql,
  pg::{Pg, PgValue},
  serialize::{Output, ToSql},
  sql_types::Jsonb,
};
use serde_json::Value;
use std::{collections::HashSet, fmt::Debug, io::Write};
use url::Url;

#[derive(
  FromSqlRow,
  PartialEq,
  Eq,
  AsExpression,
  serde::Serialize,
  serde::Deserialize,
  Debug,
  Default,
  Clone,
)]
#[diesel(sql_type = Jsonb)]
/// describes where an activity should be sent
pub struct ActivitySendTargets {
  /// send to these inboxes explicitly
  pub inboxes: HashSet<Url>,
  /// send to all followers of these local communities
  pub community_followers_of: HashSet<CommunityId>,
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
    a.add_local_community_followers(id);
    a
  }
  pub fn add_local_community_followers(&mut self, id: CommunityId) {
    self.community_followers_of.insert(id);
  }
  pub fn set_all_instances(&mut self, b: bool) {
    self.all_instances = b;
  }

  pub fn add_inbox(&mut self, inbox: Url) {
    self.inboxes.insert(inbox);
  }
  pub fn add_inboxes(&mut self, inboxes: impl Iterator<Item = Url>) {
    self.inboxes.extend(inboxes);
  }
}

#[derive(PartialEq, Eq, Debug, Queryable)]
#[diesel(table_name = sent_activity)]
pub struct SentActivity {
  pub id: i64,
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
  pub published: chrono::NaiveDateTime,
  pub send_targets: ActivitySendTargets,
  pub actor_type: ActorType,
  pub actor_apub_id: Option<DbUrl>,
}
#[derive(Insertable)]
#[diesel(table_name = sent_activity)]
pub struct SentActivityForm {
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
  pub send_targets: ActivitySendTargets,
  pub actor_type: ActorType,
  pub actor_apub_id: DbUrl,
}

#[derive(Clone, Copy, Debug, diesel_derive_enum::DbEnum, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::ActorTypeEnum"]
pub enum ActorType {
  Site,
  Community,
  Person,
}

#[derive(PartialEq, Eq, Debug, Queryable)]
#[diesel(table_name = received_activity)]
pub struct ReceivedActivity {
  pub id: i64,
  pub ap_id: DbUrl,
  pub published: chrono::NaiveDateTime,
}

// https://vasilakisfil.social/blog/2020/05/09/rust-diesel-jsonb/
impl FromSql<Jsonb, Pg> for ActivitySendTargets {
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl ToSql<Jsonb, Pg> for ActivitySendTargets {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    out.write_all(&[1])?;
    serde_json::to_writer(out, self)
      .map(|_| diesel::serialize::IsNull::No)
      .map_err(Into::into)
  }
}
