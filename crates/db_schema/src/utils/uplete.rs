use diesel::{
  associations::HasTable,
  dsl,
  pg::Pg,
  query_builder::{AsQuery, AstPass, Query, QueryFragment, QueryId},
  query_dsl::methods::{FilterDsl, SelectDsl},
  result::Error,
  sql_types,
  Column,
  Expression,
  Table,
};
use std::any::Any;
use tuplex::{IntoArray, Len};

/// Set columns to null and delete the row if all columns not in the primary key are null
pub fn uplete<Q>(query: Q) -> UpleteBuilder<Q>
where
  Q: AsQuery + Table,
  Q::Table: Default,
  Q::Query: SelectDsl<<Q::Table as Table>::PrimaryKey>,
{
  UpleteBuilder {
    query: query.as_query().select(Q::Table::default().primary_key()),
    set_null_columns: Vec::new(),
  }
}

pub struct UpleteBuilder<Q> {
  query: Q,
  set_null_columns: Vec<DynColumn>,
}

impl<Q: HasTable> UpleteBuilder<Q> {
  pub fn set_null<C: Column<Table = Q::Table> + Into<DynColumn>>(mut self, column: C) -> Self {
    self.set_null_columns.push(column.into());
    self
  }
}

impl<K0, K1, Q> AsQuery for UpleteBuilder<Q>
where
  K0: 'static,
  K1: 'static,
  (K0, K1): Expression + QueryFragment<Pg> + Send,
  Q: HasTable,
  Q::Table: Default + Table<PrimaryKey = (K0, K1)> + QueryFragment<Pg> + Send + 'static,
  <Q::Table as Table>::AllColumns: IntoArray<DynColumn>,
  <<Q::Table as Table>::AllColumns as IntoArray<DynColumn>>::Output: IntoIterator<Item = DynColumn>,
  Q: Clone + FilterDsl<AllNull> + FilterDsl<dsl::not<AllNull>>,
  dsl::Filter<Q, AllNull>: QueryFragment<Pg> + Send + 'static,
  dsl::Filter<Q, dsl::not<AllNull>>: QueryFragment<Pg> + Send + 'static,
{
  type Query = UpleteQuery;

  type SqlType = (sql_types::BigInt, sql_types::BigInt);

  fn as_query(self) -> Self::Query {
    let table = Q::Table::default();
    let pk = table.primary_key();
    let pk_type_ids = [pk.0.type_id(), pk.1.type_id()];
    let other_columns = Q::Table::all_columns()
      .into_array()
      .into_iter()
      .filter(|c: DynColumn| {
        pk_type_ids
          .iter()
          .chain(self.set_null_columns.iter().map(|c| c.type_id()))
          .all(|other| other != c.type_id())
      })
      .collect::<Vec<_>>();
    UpleteQuery {
      // Updated rows and deleted rows must not overlap, so updating all rows and using the returned
      // new rows to determine which ones to delete is not an option.
      //
      // https://www.postgresql.org/docs/16/queries-with.html#QUERIES-WITH-MODIFYING
      //
      // "Trying to update the same row twice in a single statement is not supported. Only one of
      // the modifications takes place, but it is not easy (and sometimes not possible) to reliably
      // predict which one. This also applies to deleting a row that was already updated in the same
      // statement: only the update is performed."
      update_subquery: Box::new(
        self
          .query
          .clone()
          .filter(dsl::not(AllNull(other_columns.clone()))),
      ),
      delete_subquery: Box::new(self.query.filter(AllNull(other_columns))),
      table: Box::new(table),
      primary_key: Box::new(pk),
      set_null_columns: self.set_null_columns,
    }
  }
}

pub struct UpleteQuery {
  update_subquery: Box<dyn QueryFragment<Pg> + Send + 'static>,
  delete_subquery: Box<dyn QueryFragment<Pg> + Send + 'static>,
  table: Box<dyn QueryFragment<Pg> + Send + 'static>,
  primary_key: Box<dyn QueryFragment<Pg> + Send + 'static>,
  set_null_columns: Vec<DynColumn>,
}

impl QueryId for UpleteQuery {
  type QueryId = ();

  const HAS_STATIC_QUERY_ID: bool = false;
}

impl Query for UpleteQuery {
  type SqlType = (sql_types::BigInt, sql_types::BigInt);
}

impl QueryFragment<Pg> for UpleteQuery {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    assert_ne!(self.set_null_columns.len(), 0, "`set_null` was not called");

    // Which rows to update
    out.push_sql("WITH update_keys AS (");
    self.update_subquery.walk_ast(out.reborrow())?;
    out.push_sql(" FOR UPDATE)");

    // Which rows to delete
    out.push_sql(", delete_keys AS (");
    self.delete_subquery.walk_ast(out.reborrow())?;
    out.push_sql(" FOR UPDATE)");

    // Update rows
    out.push_sql(", update_result AS (UPDATE ");
    self.table.walk_ast(out.reborrow())?;
    let mut item_prefix = " SET ";
    for column in &self.set_null_columns {
      out.push_sql(item_prefix);
      column.walk_ast(out.reborrow())?;
      out.push_sql(" = NULL");
      item_prefix = ",";
    }
    out.push_sql(" WHERE (");
    self.primary_key.walk_ast(out.reborrow())?;
    out.push_sql(") = ANY (SELECT * FROM update_keys) RETURNING 1)");

    // Delete rows
    out.push_sql(", delete_result AS (DELETE FROM ");
    self.table.walk_ast(out.reborrow())?;
    out.push_sql(" WHERE (");
    self.primary_key.walk_ast(out.reborrow())?;
    out.push_sql(") = ANY (SELECT * FROM update_keys) RETURNING 1)");

    // Count updated rows and deleted rows (`RETURNING 1` makes this possible)
    out.push_sql(" SELECT (SELECT count(*) FROM update_result)");
    out.push_sql(", (SELECT count(*) FROM delete_result)");

    Ok(())
  }
}

pub struct AllNull(Vec<DynColumn>);

impl Expression for AllNull {
  type SqlType = sql_types::Bool;
}

impl QueryFragment<Pg> for AllNull {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    let mut item_prefix = "(";
    for column in &self.0 {
      out.push_sql(item_prefix);
      column.walk_ast(out.reborrow())?;
      out.push_sql(" IS NOT NULL");
      item_prefix = " AND ";
    }
    out.push_sql(")");

    Ok(())
  }
}

pub struct DynColumn(Box<dyn QueryFragment<Pg> + Send + 'static>);

impl<T: QueryFragment<Pg> + 'static> From<T> for DynColumn {
  fn from(value: T) -> Self {
    DynColumn(Box::new(value))
  }
}

#[derive(Queryable, PartialEq, Eq, Debug)]
pub struct UpleteCount {
  pub updated: i64,
  pub deleted: i64,
}

impl UpleteCount {
  pub fn only_updated(n: i64) -> Self {
    UpleteCount {
      updated: n,
      deleted: 0,
    }
  }

  pub fn only_deleted(n: i64) -> Self {
    UpleteCount {
      updated: 0,
      deleted: n,
    }
  }
}
