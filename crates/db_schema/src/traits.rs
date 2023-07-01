use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
  utils::DbConn,
};
use diesel::result::Error;

#[async_trait]
pub trait Crud {
  type InsertForm;
  type UpdateForm;
  type IdType;
  async fn create(mut conn: impl DbConn, form: &Self::InsertForm) -> Result<Self, Error>
  where
    Self: Sized;
  async fn read(mut conn: impl DbConn, id: Self::IdType) -> Result<Self, Error>
  where
    Self: Sized;
  /// when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  async fn update(
    mut conn: impl DbConn,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn delete(_conn: impl DbConn, _id: Self::IdType) -> Result<usize, Error>
  where
    Self: Sized,
    Self::IdType: Send,
  {
    async { Err(Error::NotFound) }.await
  }
}

#[async_trait]
pub trait Followable {
  type Form;
  async fn follow(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn follow_accepted(
    mut conn: impl DbConn,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unfollow(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Joinable {
  type Form;
  async fn join(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn leave(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Likeable {
  type Form;
  type IdType;
  async fn like(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn remove(
    mut conn: impl DbConn,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Bannable {
  type Form;
  async fn ban(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unban(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Saveable {
  type Form;
  async fn save(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unsave(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Blockable {
  type Form;
  async fn block(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unblock(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Readable {
  type Form;
  async fn mark_as_read(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn mark_as_unread(mut conn: impl DbConn, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Reportable {
  type Form;
  type IdType;
  async fn report(mut conn: impl DbConn, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn resolve(
    mut conn: impl DbConn,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
  async fn unresolve(
    mut conn: impl DbConn,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait JoinView {
  type JoinTuple;
  fn from_tuple(tuple: Self::JoinTuple) -> Self
  where
    Self: Sized;
}

#[async_trait]
pub trait ApubActor {
  async fn read_from_apub_id(
    mut conn: impl DbConn,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error>
  where
    Self: Sized;
  /// - actor_name is the name of the community or user to read.
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  async fn read_from_name(
    mut conn: impl DbConn,
    actor_name: &str,
    include_deleted: bool,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn read_from_name_and_domain(
    mut conn: impl DbConn,
    actor_name: &str,
    protocol_domain: &str,
  ) -> Result<Self, Error>
  where
    Self: Sized;
}
