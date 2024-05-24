use diesel::{
  dsl,
  expression::{is_aggregate, NonAggregate, ValidGrouping},
  pg::Pg,
  query_builder::{AsQuery, AstPass, QueryFragment, UpdateStatement},
  result::Error,
  sql_types,
  AppearsOnTable,
  Expression,
  Insertable,
  QueryId,
  SelectableExpression,
  Table,
};

pub trait UpleteTable: Table + Default {
  type EmptyRow: Default + AsExpression<sql_types::Record<Table::SqlType>>;
}

pub trait OrDelete {
  type Output;

  fn or_delete() -> Self::Output;
}

impl<T: UpleteTable, U, V> OrDelete for UpdateStatement<T, U, V> {
  type Output = SetOrDeleteQuery<T, T::PrimaryKey, T::AllColumns, V, T::EmptyRow>;

  fn or_delete() -> Self::Output {
    SetOrDeleteQuery {
      table: T::default(),
      primary_key: T::primary_key(),
      all_columns: T::all_columns(),
      update_values: 
    }
  }
}

impl<T: Table, U, E> Uplete<T, T::PrimaryKey, T::AllColumns, U, E>

#[derive(QueryId)]
struct SetOrDeleteQuery<T, PK, C, U, E> {
  table: T,
  primary_key: PK,
  all_columns: C,
  update_statement: U,
  empty_row: E,
}

impl<T: QueryFragment<Pg>, PK: QueryFragment<Pg>, C: QueryFragment<Pg>, U: QueryFragment<Pg>, E: QueryFragment<Pg>> QueryFragment<Pg> for SetOrDeleteQuery<T, U, E> {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    out.push_sql("WITH update_result AS (");
    self.update_statement.walk_ast(out.reborrow())?;
    self.push_sql(" RETURNING ");
    self.all_columns.walk_ast(out.reborrow())?;
    self.push_sql(") DELETE FROM ");
    self.table.walk_ast(out.reborrow())?;
    self.push_sql(" WHERE (");
    self.primary_key.walk_ast(out.reborrow())?;
    self.push_sql(") = ANY (SELECT (");
    // In the parts below, `self.table` refers to `update_result`
    self.primary_key.walk_ast(out.reborrow())?;
    self.push_sql(") FROM update_result AS ");
    self.table.walk_ast(out.reborrow())?;
    self.push_sql(" WHERE (");
    self.all_columns.walk_ast(out.reborrow())?;
    self.push_sql(") = (");
    self.empty_row.walk_ast(out.reborrow())?;
    self.push_sql("))");
  }
}
