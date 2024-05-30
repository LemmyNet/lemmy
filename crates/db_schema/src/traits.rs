use crate::{
  diesel::OptionalExtension,
  newtypes::{CommunityId, DbUrl, PersonId},
  utils::{get_conn, uplete::uplete::Count, DbPool},
};
use diesel::{
  associations::HasTable,
  dsl,
  query_builder::{DeleteStatement, IntoUpdateTarget},
  query_dsl::methods::{FindDsl, LimitDsl},
  result::Error,
  Table,
};
use diesel_async::{
  methods::{ExecuteDsl, LoadQuery},
  AsyncPgConnection,
  RunQueryDsl,
};

/// Returned by `diesel::delete`
pub type Delete<T> = DeleteStatement<<T as HasTable>::Table, <T as IntoUpdateTarget>::WhereClause>;

/// Returned by `Self::table().find(id)`
pub type Find<T> = dsl::Find<<T as HasTable>::Table, <T as Crud>::IdType>;

pub type PrimaryKey<T> = <<T as HasTable>::Table as Table>::PrimaryKey;

// Trying to create default implementations for `create` and `update` results in a lifetime mess and
// weird compile errors. https://github.com/rust-lang/rust/issues/102211
#[async_trait]
pub trait Crud: HasTable + Sized
where
  Self::Table: FindDsl<Self::IdType>,
  Find<Self>: LimitDsl + IntoUpdateTarget + Send,
  Delete<Find<Self>>: ExecuteDsl<AsyncPgConnection> + Send + 'static,

  // Used by `RunQueryDsl::first`
  dsl::Limit<Find<Self>>: LoadQuery<'static, AsyncPgConnection, Self> + Send + 'static,
{
  type InsertForm;
  type UpdateForm;
  type IdType: Send;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error>;

  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> Result<Option<Self>, Error> {
    let query: Find<Self> = Self::table().find(id);
    let conn = &mut *get_conn(pool).await?;
    query.first(conn).await.optional()
  }

  /// when you want to null out a column, you have to send Some(None)), since sending None means you
  /// just don't want to update that column.
  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error>;

  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> Result<usize, Error> {
    let query: Delete<Find<Self>> = diesel::delete(Self::table().find(id));
    let conn = &mut *get_conn(pool).await?;
    query.execute(conn).await
  }
}

#[async_trait]
pub trait Followable {
  type Form;
  async fn follow(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unfollow(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Joinable {
  type Form;
  async fn join(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn leave(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Likeable {
  type Form;
  type IdType;
  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn remove(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Bannable {
  type Form;
  async fn ban(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unban(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Saveable {
  type Form;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Blockable {
  type Form;
  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<uplete::Count, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Reportable {
  type Form;
  type IdType;
  type ObjectIdType;
  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    comment_id_: Self::ObjectIdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait ApubActor {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error>
  where
    Self: Sized;
  /// - actor_name is the name of the community or user to read.
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  async fn read_from_name(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    include_deleted: bool,
  ) -> Result<Option<Self>, Error>
  where
    Self: Sized;
  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    protocol_domain: &str,
  ) -> Result<Option<Self>, Error>
  where
    Self: Sized;
}
