use crate::newtypes::{CommunityId, DbUrl, PersonId};
use diesel::{result::Error, PgConnection};

pub trait Crud {
  type Form;
  type IdType;
  fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn read(conn: &mut PgConnection, id: Self::IdType) -> Result<Self, Error>
  where
    Self: Sized;
  fn update(conn: &mut PgConnection, id: Self::IdType, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn delete(_conn: &mut PgConnection, _id: Self::IdType) -> Result<usize, Error>
  where
    Self: Sized,
  {
    unimplemented!()
  }
}

pub trait Followable {
  type Form;
  fn follow(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn follow_accepted(
    conn: &mut PgConnection,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  fn unfollow(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
  fn has_local_followers(conn: &mut PgConnection, community_id: CommunityId)
    -> Result<bool, Error>;
}

pub trait Joinable {
  type Form;
  fn join(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn leave(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Likeable {
  type Form;
  type IdType;
  fn like(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn remove(
    conn: &mut PgConnection,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Bannable {
  type Form;
  fn ban(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn unban(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Saveable {
  type Form;
  fn save(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn unsave(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Blockable {
  type Form;
  fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Readable {
  type Form;
  fn mark_as_read(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn mark_as_unread(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Reportable {
  type Form;
  type IdType;
  fn report(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  fn resolve(
    conn: &mut PgConnection,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
  fn unresolve(
    conn: &mut PgConnection,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait DeleteableOrRemoveable {
  fn blank_out_deleted_or_removed_info(self) -> Self;
}

pub trait ToSafe {
  type SafeColumns;
  fn safe_columns_tuple() -> Self::SafeColumns;
}

pub trait ToSafeSettings {
  type SafeSettingsColumns;
  fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns;
}

pub trait ViewToVec {
  type DbTuple;
  fn from_tuple_to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
  where
    Self: Sized;
}

pub trait ApubActor {
  // TODO: this should be in a trait ApubObject (and implemented for Post, Comment, PrivateMessage as well)
  fn read_from_apub_id(conn: &mut PgConnection, object_id: &DbUrl) -> Result<Option<Self>, Error>
  where
    Self: Sized;
  /// - actor_name is the name of the community or user to read.
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  fn read_from_name(
    conn: &mut PgConnection,
    actor_name: &str,
    include_deleted: bool,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  fn read_from_name_and_domain(
    conn: &mut PgConnection,
    actor_name: &str,
    protocol_domain: &str,
  ) -> Result<Self, Error>
  where
    Self: Sized;
}
