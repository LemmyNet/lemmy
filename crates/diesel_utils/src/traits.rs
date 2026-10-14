use crate::connection::{DbPool, get_conn};
use diesel::{
  associations::HasTable,
  dsl,
  query_builder::{DeleteStatement, IntoUpdateTarget},
  query_dsl::methods::{FindDsl, LimitDsl},
};
use diesel_async::{
  AsyncPgConnection,
  RunQueryDsl,
  methods::{ExecuteDsl, LoadQuery},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::future::Future;

/// Returned by `diesel::delete`
type Delete<T> = DeleteStatement<<T as HasTable>::Table, <T as IntoUpdateTarget>::WhereClause>;

/// Returned by `Self::table().find(id)`
type Find<T> = dsl::Find<<T as HasTable>::Table, <T as Crud>::IdType>;

// Trying to create default implementations for `create` and `update` results in a lifetime mess and
// weird compile errors. https://github.com/rust-lang/rust/issues/102211
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

  fn create(
    pool: &mut DbPool<'_>,
    form: &Self::InsertForm,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;

  fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Send,
  {
    async {
      let query: Find<Self> = Self::table().find(id);
      let conn = &mut *get_conn(pool).await?;
      query
        .first(conn)
        .await
        .with_lemmy_type(LemmyErrorType::NotFound)
    }
  }

  /// when you want to null out a column, you have to send Some(None)), since sending None means you
  /// just don't want to update that column.
  fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;

  fn delete(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<usize>> + Send {
    async {
      let query: Delete<Find<Self>> = diesel::delete(Self::table().find(id));
      let conn = &mut *get_conn(pool).await?;
      query
        .execute(conn)
        .await
        .with_lemmy_type(LemmyErrorType::Deleted)
    }
  }
}
