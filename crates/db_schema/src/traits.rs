use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
  utils::DbPool,
  DbPoolRef,
};
use diesel::result::Error;

#[async_trait]
pub trait Crud {
  type InsertForm;
  type UpdateForm;
  type IdType;
  async fn create(pool: DbPoolRef<'_>, form: &Self::InsertForm) -> Result<Self, Error>
  where
    Self: Sized;
  async fn read(pool: DbPoolRef<'_>, id: Self::IdType) -> Result<Self, Error>
  where
    Self: Sized;
  /// when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  async fn update(
    pool: DbPoolRef<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn delete(_pool: DbPoolRef<'_>, _id: Self::IdType) -> Result<usize, Error>
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
  async fn follow(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn follow_accepted(
    pool: DbPoolRef<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unfollow(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Joinable {
  type Form;
  async fn join(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn leave(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Likeable {
  type Form;
  type IdType;
  async fn like(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn remove(
    pool: DbPoolRef<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Bannable {
  type Form;
  async fn ban(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unban(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Saveable {
  type Form;
  async fn save(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unsave(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Blockable {
  type Form;
  async fn block(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unblock(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Readable {
  type Form;
  async fn mark_as_read(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn mark_as_unread(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Reportable {
  type Form;
  type IdType;
  async fn report(pool: DbPoolRef<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn resolve(
    pool: DbPoolRef<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
  async fn unresolve(
    pool: DbPoolRef<'_>,
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
  async fn read_from_apub_id(pool: DbPoolRef<'_>, object_id: &DbUrl) -> Result<Option<Self>, Error>
  where
    Self: Sized;
  /// - actor_name is the name of the community or user to read.
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  async fn read_from_name(
    pool: DbPoolRef<'_>,
    actor_name: &str,
    include_deleted: bool,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn read_from_name_and_domain(
    pool: DbPoolRef<'_>,
    actor_name: &str,
    protocol_domain: &str,
  ) -> Result<Self, Error>
  where
    Self: Sized;
}
