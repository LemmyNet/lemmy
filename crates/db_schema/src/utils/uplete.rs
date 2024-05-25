use diesel::{
  dsl,
  expression::{AsExpression, TypedExpressionType},
  pg::Pg,
  query_builder::{AstPass, Query, QueryFragment, UpdateStatement},
  result::Error,
  sql_types,
  QueryId,
  Table,
};

pub trait UpleteTable: Table + Default {
  type EmptyRow;
}

pub trait OrDelete {
  type Output;

  /// Change an update query so rows that equal `UpleteTable::EmptyRow::default()` are deleted
  fn or_delete(self) -> Self::Output;
}

impl<T: UpleteTable, U, V> OrDelete for UpdateStatement<T, U, V>
where
  T::SqlType: sql_types::SqlType + TypedExpressionType,
  T::EmptyRow: Default + AsExpression<sql_types::Record<T::SqlType>>,
{
  type Output = SetOrDeleteQuery<
    T,
    T::PrimaryKey,
    T::AllColumns,
    Self,
    dsl::AsExprOf<T::EmptyRow, sql_types::Record<T::SqlType>>,
  >;

  fn or_delete(self) -> Self::Output {
    SetOrDeleteQuery {
      table: T::default(),
      primary_key: T::default().primary_key(),
      all_columns: T::all_columns(),
      update_statement: self,
      empty_row: T::EmptyRow::default().as_expression(),
    }
  }
}

#[derive(QueryId)]
pub struct SetOrDeleteQuery<T, PK, C, U, E> {
  table: T,
  primary_key: PK,
  all_columns: C,
  update_statement: U,
  empty_row: E,
}

impl<T, PK, C, U, E> Query for SetOrDeleteQuery<T, PK, C, U, E> {
  type SqlType = (sql_types::BigInt, sql_types::BigInt);
}

impl<
    T: QueryFragment<Pg>,
    PK: QueryFragment<Pg>,
    C: QueryFragment<Pg>,
    U: QueryFragment<Pg>,
    E: QueryFragment<Pg>,
  > QueryFragment<Pg> for SetOrDeleteQuery<T, PK, C, U, E>
{
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    // `update_result` CTE with new rows (concurrent writers to these rows are blocked until this query ends)
    out.push_sql("WITH update_result AS (");
    self.update_statement.walk_ast(out.reborrow())?;
    out.push_sql(" RETURNING ");
    self.all_columns.walk_ast(out.reborrow())?;

    // Beginning of `delete_result` CTE with 1 row per deleted row
    out.push_sql("), delete_result AS (DELETE FROM ");
    self.table.walk_ast(out.reborrow())?;
    out.push_sql(" WHERE (");
    self.primary_key.walk_ast(out.reborrow())?;

    // Select from `update_result` with an alias that matches the original table's name
    out.push_sql(") = ANY (SELECT ");
    self.primary_key.walk_ast(out.reborrow())?;
    out.push_sql(" FROM update_result AS ");
    self.table.walk_ast(out.reborrow())?;

    // Filter the select statement
    out.push_sql(" WHERE (");
    self.all_columns.walk_ast(out.reborrow())?;
    out.push_sql(") IS NOT DISTINCT FROM ");
    self.empty_row.walk_ast(out.reborrow())?;

    // Select count from each CTE
    out.push_sql(") RETURNING 1) SELECT (SELECT count(*) from update_result), (SELECT count(*) FROM delete_result)");

    Ok(())
  }
}

#[derive(Queryable, PartialEq, Eq, Debug)]
pub struct UpleteCount {
  pub all: i64,
  pub deleted: i64,
}

impl UpleteCount {
  pub fn only_updated(n: i64) -> Self {
    UpleteCount {
      all: n,
      deleted: 0,
    }
  }

  pub fn only_deleted(n: i64) -> Self {
    UpleteCount {
      all: n,
      deleted: n,
    }
  }
}
