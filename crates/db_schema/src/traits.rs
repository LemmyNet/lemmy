use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
  utils::{get_conn, DbPool},
};
use diesel::{
  associations::HasTable,
  dsl,
  expression::{AsExpression, TypedExpressionType},
  expression_methods::ExpressionMethods,
  insert_into,
  query_builder::{DeleteStatement, InsertStatement, IntoUpdateTarget},
  query_dsl::methods::{FindDsl, LimitDsl},
  result::Error,
  sql_types::SqlType,
  AsChangeset,
  Expression,
  Insertable,
  Table,
};
use diesel_async::{
  methods::{ExecuteDsl, LoadQuery},
  AsyncPgConnection,
  RunQueryDsl,
};
use futures_util::{Future, TryFutureExt};
use std::{hash::Hash, pin::Pin};

/// Returned by `diesel::delete`
pub type Delete<T> = DeleteStatement<<T as HasTable>::Table, <T as IntoUpdateTarget>::WhereClause>;

pub type Find<T, IdType> = dsl::Find<<T as HasTable>::Table, IdType>;

pub type InsertValues<'a, T, InsertForm> =
  <&'a InsertForm as Insertable<<T as HasTable>::Table>>::Values;

pub trait CrudBounds<'a, InsertForm, UpdateForm, IdType>
where
  Self: HasTable + Sized,
  Self::Table: FindDsl<IdType> + 'static,
  Find<Self, IdType>: LimitDsl + Send + IntoUpdateTarget + 'static,
  dsl::Limit<Find<Self, IdType>>: Send + LoadQuery<'static, AsyncPgConnection, Self> + 'static,
  <Self::Table as Table>::PrimaryKey: ExpressionMethods + Send,
  <<Self::Table as Table>::PrimaryKey as Expression>::SqlType: SqlType + TypedExpressionType,
  Delete<Find<Self, IdType>>: ExecuteDsl<AsyncPgConnection> + Send + 'static,
  <Find<Self, IdType> as IntoUpdateTarget>::WhereClause: 'static + Send,
  <Find<Self, IdType> as HasTable>::Table: 'static + Send,
  &'a InsertForm: Insertable<Self::Table>,
  InsertValues<'a, Self, InsertForm>: 'a,
  InsertStatement<Self::Table, InsertValues<'a, Self, InsertForm>>:
    LoadQuery<'a, AsyncPgConnection, Self> + 'a + Send,
  InsertForm: 'static + Send + Sync,
  UpdateForm: 'static + Send + Sync,
  IdType: 'static
    + Hash
    + Eq
    + Sized
    + Send
    + AsExpression<<<Self::Table as Table>::PrimaryKey as Expression>::SqlType>,
{
}

impl<'a, InsertForm, UpdateForm, IdType, T> CrudBounds<'a, InsertForm, UpdateForm, IdType> for T
where
  Self: HasTable + Sized,
  Self::Table: FindDsl<IdType> + 'static,
  Find<Self, IdType>: LimitDsl + Send + IntoUpdateTarget + 'static,
  dsl::Limit<Find<Self, IdType>>: Send + LoadQuery<'static, AsyncPgConnection, Self> + 'static,
  <Self::Table as Table>::PrimaryKey: ExpressionMethods + Send,
  <<Self::Table as Table>::PrimaryKey as Expression>::SqlType: SqlType + TypedExpressionType,
  Delete<Find<Self, IdType>>: ExecuteDsl<AsyncPgConnection> + Send + 'static,
  <Find<Self, IdType> as IntoUpdateTarget>::WhereClause: 'static + Send,
  <Find<Self, IdType> as HasTable>::Table: 'static + Send,
  &'a InsertForm: Insertable<Self::Table>,
  InsertValues<'a, Self, InsertForm>: 'a,
  InsertStatement<Self::Table, InsertValues<'a, Self, InsertForm>>:
    LoadQuery<'a, AsyncPgConnection, Self> + 'a + Send,
  InsertForm: 'static + Send + Sync,
  UpdateForm: 'static + Send + Sync,
  IdType: 'static
    + Hash
    + Eq
    + Sized
    + Send
    + AsExpression<<<Self::Table as Table>::PrimaryKey as Expression>::SqlType>,
{
}

// When using `RunQueryDsl::execute`, directly building futures with `Box::pin` and `TryFutureExt::and_then`
// instead of `async` + `await` fixes weird compile errors.
// https://github.com/rust-lang/rust/issues/102211
// When using `RunQueryDsl::first` or 'RunQueryDsl::get_result`, `async` + `await` works, and it must be used otherwise the closure for `and_then`
// will both own `conn` and return a future that references it.
#[async_trait]
pub trait Crud<'a>
where
  for<'b> Self: CrudBounds<'b, Self::InsertForm, Self::UpdateForm, Self::IdType>,
  Self: Sized,
{
  type InsertForm: 'static + Send + Sync;
  type UpdateForm: 'static + Send + Sync;
  type IdType: 'static
    + Hash
    + Eq
    + Sized
    + Send
    + AsExpression<<<Self::Table as Table>::PrimaryKey as Expression>::SqlType>;
  async fn create<'life0, 'life1>(
    pool: &'life0 mut DbPool<'life1>,
    form: &'a Self::InsertForm,
  ) -> Result<Self, Error>
  //Pin<Box<dyn Future<Output = Result<Self, Error>> + Send + 'async_trait>>
  where
    'a: 'async_trait,
  {
    let query = insert_into(Self::table()).values(form);
    let conn = &mut *get_conn(pool).await?;
    query.get_result::<Self>(conn).await
    //Box::pin(get_conn(pool).and_then(move |mut conn| query.get_result::<Self>(&mut *conn)))
  }
  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> Result<Self, Error>
  where
    'a: 'async_trait,
  {
    let query = Self::table().find(id);
    let conn = &mut *get_conn(pool).await?;
    query.first::<Self>(conn).await
  }
  /// when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &'a Self::UpdateForm,
  ) -> Result<Self, Error>
  where
    'a: 'async_trait;
  /*{
    let conn = &mut get_conn(pool).await?;
    diesel::update(Self::table().find(id))
    .set(form)
    .get_result::<Self>(conn)
    .await
  }*/
  fn delete<'life0, 'life1, 'async_trait>(
    pool: &'life0 mut DbPool<'life1>,
    id: Self::IdType,
  ) -> Pin<Box<dyn Future<Output = Result<usize, Error>> + Send + 'async_trait>>
  where
    'a: 'async_trait,
    'life0: 'async_trait,
    'life1: 'async_trait,
    Self: Send + 'async_trait,
  {
    let query: Delete<Find<'a, Self>> = diesel::delete(Self::table().find(id));
    Box::pin(get_conn(pool).and_then(move |mut conn| query.execute(&mut *conn)))
  }
}

#[async_trait]
pub trait Followable
where
  Self: HasTable,
  for<'a> &'a Self::Form: AsChangeset<Target = Self::Table> + Insertable<Self::Table>,
{
  //type FollowerColumn: Column + Default + Send;
  //type TargetColumn: Column + Default + Send;
  type Form;
  async fn follow(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  /*{
    let conn = &mut get_conn(pool).await?;
    insert_into(Self::table())
      .values(form)
      .on_conflict((
        Self::TargetColumn::default(),
        Self::FollowerColumn::default(),
      ))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }*/
  async fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unfollow(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Joinable {
  type Form;
  async fn join(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn leave(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
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
  ) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Bannable {
  type Form;
  async fn ban(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unban(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Saveable {
  type Form;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Blockable {
  type Form;
  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Readable {
  type Form;
  async fn mark_as_read(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<Self, Error>
  where
    Self: Sized;
  async fn mark_as_unread(pool: &mut DbPool<'_>, form: &Self::Form) -> Result<usize, Error>
  where
    Self: Sized;
}

#[async_trait]
pub trait Reportable {
  type Form;
  type IdType;
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
  async fn unresolve(
    pool: &mut DbPool<'_>,
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
  ) -> Result<Self, Error>
  where
    Self: Sized;
  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    protocol_domain: &str,
  ) -> Result<Self, Error>
  where
    Self: Sized;
}
