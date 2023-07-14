use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
  utils::{get_conn, DbPool, DbConn},
};
use diesel::{
  associations::HasTable,
  backend::Backend,
  deserialize::{FromSqlRow, Queryable},
  dsl::insert_into,
  pg::Pg,
  query_builder::{AsQuery, IntoUpdateTarget, Query, QueryFragment, QueryId,Only},
  result::Error,
  AsChangeset,
  Column,
  Expression,
  Identifiable,
  Insertable,
  QuerySource,
  Table,
  query_dsl::methods::{SelectDsl,LimitDsl,FilterDsl},
  query_source::{Once,AppearsInFromClause},
  expression::{ValidGrouping,is_aggregate, IsContainedInGroupBy,is_contained_in_group_by},
  expression_methods::ExpressionMethods,
  sql_types::{SqlType,is_nullable::NotNull},
  expression::AsExpression,
  SelectableExpression,
  dsl,
};
use diesel_async::{AsyncConnection, RunQueryDsl,methods::LoadQuery};
use std::hash::Hash;

#[async_trait]
pub trait Crud
where
  /*Self: Send + HasTable + Send + 'static + FromSqlRow<<Self::Table as AsQuery>::SqlType, Pg>,
  Self::Table: Send + FindDsl<Self::IdType>,
  <Self::Table as FindDsl<Self::IdType>>::Output: LimitDsl + Send,
  for<'conn> <<Self::Table as FindDsl<Self::IdType>>::Output as LimitDsl>::Output:
    Send + LoadQuery<'static, DbConn<'conn>, Self> /*+ 'query*/ + Send,
  <Self::Table as AsQuery>::SqlType: Send,*/

  /*Self: Send + 'static + HasTable + Sized,
  Self::Table: Send + 'static + AppearsInFromClause<Only<Self::Table>, Count = Once> + QueryFragment<Pg> + HasTable<Table = Self::Table> + IntoUpdateTarget<WhereClause = <<Self::Table as AsQuery>::Query as IntoUpdateTarget>::WhereClause> + AppearsInFromClause<Self::Table, Count = Once>,
  <Self::Table as AsQuery>::Query: IntoUpdateTarget,
  //Only<Self::Table>: AppearsInFromClause<Only<Self::Table>, Count = Once>,
  <Self::Table as Table>::PrimaryKey: Send + 'static + QueryFragment<Pg> + ValidGrouping<(), IsAggregate = is_aggregate::No> + /*IsContainedInGroupBy<<Self::Table as Table>::PrimaryKey, Output = is_contained_in_group_by::Yes> +*/ Column<Table = Self::Table> + AppearsInFromClause<Only<Self::Table>> + SelectableExpression<Only<Self::Table>> + Expression + Sized,
  <<Self::Table as Table>::PrimaryKey as Expression>::SqlType: SqlType<IsNull = NotNull>,*/

  Self: Send + 'static + Sized + HasTable,
  Self::Table: FilterDsl<dsl::Eq<<Self::Table as Table>::PrimaryKey, Self::IdType>> + Send + Sized + 'static,
  <Self::Table as FilterDsl<dsl::Eq<<Self::Table as Table>::PrimaryKey, Self::IdType>>>::Output: LimitDsl + Send + Sized + 'static,
  //for<'conn> diesel::helper_types::Limit<<Self::Table as FilterDsl<dsl::Eq<<Self::Table as Table>::PrimaryKey, Self::IdType>>>::Output>: LoadQuery<'static, DbConn<'conn>, Self> + Send + 'static + Sized,
  <<Self::Table as Table>::PrimaryKey as Expression>::SqlType: SqlType,
  <Self::Table as Table>::PrimaryKey: ExpressionMethods + Send + Sized + 'static,
  //<Self::Table as Table>::AllColumns: Send + Sized + 'static,
  //<Self::Table as AsQuery>::Query: Send + Sized + 'static,

  //<Self::Table as FilterDsl<dsl::Eq<<Self::Table as Table>::PrimaryKey, Self::IdType>>>::Output: 
{
  type InsertForm;
  type UpdateForm;
  type IdType: Hash + Eq + Send + 'static + Sized + AsExpression<<<Self::Table as Table>::PrimaryKey as Expression>::SqlType>;
  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error>; /*{
                   let conn = &mut get_conn(pool).await?;
                   insert_into(Self::table())
                     .values(form)
                     .get_result::<Self>(conn)
                     .await
                 }*/
  async fn read(pool: &'async_trait mut DbPool<'_>, id: Self::IdType) -> Result<Self, Error>
  where diesel::helper_types::Limit<<Self::Table as FilterDsl<dsl::Eq<<Self::Table as Table>::PrimaryKey, Self::IdType>>>::Output>: LoadQuery<'static, DbConn<'async_trait>, Self> + Send + 'static + Sized
  {
    let col = Self::table().primary_key();
    // FindDsl is not used because it uses a private trait
    let query = FilterDsl::filter(Self::table(), ExpressionMethods::eq(col, id));
    let mut conn = get_conn(pool).await?;
    let future = RunQueryDsl::first::<'static, 'async_trait, Self>(query, &mut conn);
    future.await
  }
  /// when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error>; /*{
                   let conn = &mut get_conn(pool).await?;
                   diesel::update(Self::table().find(id))
                     .set(form)
                     .get_result::<Self>(conn)
                     .await
                 }*/
  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> Result<usize, Error>
  {
    Err(Error::NotFound)
    /*let conn = &mut get_conn(pool).await?;
    diesel::delete(Self::table().find(id)).execute(conn).await*/
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
    Self: Sized; /*{
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
