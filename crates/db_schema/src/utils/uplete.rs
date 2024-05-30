use diesel::{
  associations::HasTable,
  dsl,
  expression::{is_aggregate, ValidGrouping},
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
use tuplex::IntoArray;

/// Set columns (each specified with `UpleteBuilder::set_null`) to null in the rows found by
/// `query`, and delete rows that have no remaining non-null values outside of the primary key
pub fn new<Q>(query: Q) -> UpleteBuilder<dsl::Select<Q::Query, <Q::Table as Table>::PrimaryKey>>
where
  Q: AsQuery + HasTable,
  Q::Table: Default,
  Q::Query: SelectDsl<<Q::Table as Table>::PrimaryKey>,

  // For better error messages
  UpleteBuilder<Q>: AsQuery,
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

impl<Q> AsQuery for UpleteBuilder<Q>
where
  Q: HasTable,
  Q::Table: Default + QueryFragment<Pg> + Send + 'static,
  <Q::Table as Table>::PrimaryKey: IntoArray<DynColumn> + QueryFragment<Pg> + Send + 'static,
  <Q::Table as Table>::AllColumns: IntoArray<DynColumn>,
  <<Q::Table as Table>::PrimaryKey as IntoArray<DynColumn>>::Output: IntoIterator<Item = DynColumn>,
  <<Q::Table as Table>::AllColumns as IntoArray<DynColumn>>::Output: IntoIterator<Item = DynColumn>,
  Q: Clone + FilterDsl<AllNull> + FilterDsl<dsl::not<AllNull>>,
  dsl::Filter<Q, AllNull>: QueryFragment<Pg> + Send + 'static,
  dsl::Filter<Q, dsl::not<AllNull>>: QueryFragment<Pg> + Send + 'static,
{
  type Query = UpleteQuery;

  type SqlType = (sql_types::BigInt, sql_types::BigInt);

  fn as_query(self) -> Self::Query {
    let table = Q::Table::default();
    let deletion_condition = || {
      AllNull(
        Q::Table::all_columns()
          .into_array()
          .into_iter()
          .filter(|c: DynColumn| {
            table
              .primary_key()
              .into_array()
              .into_iter()
              .chain(&self.set_null_columns)
              .all(|excluded_column| excluded_column.type_id() != c.type_id())
          })
          .collect::<Vec<_>>(),
      )
    };
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
      update_subquery: Box::new(self.query.clone().filter(dsl::not(deletion_condition()))),
      delete_subquery: Box::new(self.query.filter(deletion_condition())),
      table: Box::new(table),
      primary_key: Box::new(table.primary_key()),
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

    // Declare `update_keys` and `delete_keys` CTEs, which select primary keys
    for (prefix, subquery) in [
      ("WITH update_keys", &self.update_subquery),
      (", delete_keys", &self.delete_subquery),
    ] {
      out.push_sql(prefix);
      out.push_sql(" AS (");
      subquery.walk_ast(out.reborrow())?;
      out.push_sql(" FOR UPDATE)");
    }

    // Update rows that are referenced in `update_keys`
    out.push_sql(", update_result AS (UPDATE ");
    self.table.walk_ast(out.reborrow())?;
    let mut item_prefix = " SET ";
    for column in &self.set_null_columns {
      out.push_sql(item_prefix);
      column.0.walk_ast(out.reborrow())?;
      out.push_sql(" = NULL");
      item_prefix = ",";
    }
    out.push_sql(" WHERE (");
    self.primary_key.walk_ast(out.reborrow())?;
    out.push_sql(") = ANY (SELECT * FROM update_keys) RETURNING 1)");

    // Delete rows that are referenced in `delete_keys`
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

impl ValidGrouping<()> for AllNull {
  type IsAggregate = is_aggregate::No;
}

impl QueryFragment<Pg> for AllNull {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    let mut item_prefix = "(";
    for column in &self.0 {
      out.push_sql(item_prefix);
      column.0.walk_ast(out.reborrow())?;
      out.push_sql(" IS NOT NULL");
      item_prefix = " AND ";
    }
    out.push_sql(")");

    Ok(())
  }
}

pub struct DynColumn(Box<dyn QueryFragment<Pg> + Send + 'static>);

impl<T: QueryFragment<Pg> + Send + 'static> From<T> for DynColumn {
  fn from(value: T) -> Self {
    DynColumn(Box::new(value))
  }
}

#[derive(Queryable, PartialEq, Eq, Debug)]
pub struct Count {
  pub updated: i64,
  pub deleted: i64,
}

impl Count {
  pub fn only_updated(n: i64) -> Self {
    Count {
      updated: n,
      deleted: 0,
    }
  }

  pub fn only_deleted(n: i64) -> Self {
    Count {
      updated: 0,
      deleted: n,
    }
  }
}
