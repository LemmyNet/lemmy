use diesel::{
  dsl,
  expression::{is_aggregate, ValidGrouping},
  pg::Pg,
  query_builder::{AsQuery, AstPass, QueryFragment},
  result::Error,
  sql_types,
  AppearsOnTable,
  Expression,
  Insertable,
  QueryId,
  SelectableExpression,
  Table,
};

#[derive(QueryId)]
pub struct Uplete<T, U, E> {
  pub table: T,
  pub update_values: U,
  pub empty_row: E,
}

impl<T: QueryFragment<Pg> + Table, U: QueryFragment<Pg>, E: QueryFragment<Pg>> QueryFragment<Pg> for Uplete<T, U, E> {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    out.push_sql("WITH update_result AS (UPDATE ");
    self.update_values.walk_ast(out.reborrow())?;
    self.push_sql(" RETURNING ");
    self.table.all_columns().walk_ast(out.reborrow())?;
    self.push_sql(") DELETE FROM ");
    self.table.walk_ast(out.reborrow())?;
    self.push_sql(" WHERE (");
    self.table.primary_key().walk_ast(out.reborrow())?;
    self.push_sql(") = ANY (SELECT (");
    // In the parts below, `self.table` refers to `update_result`
    self.table.primary_key().walk_ast(out.reborrow())?;
    self.push_sql(") FROM update_result AS ");
    self.table.walk_ast(out.reborrow())?;
    self.push_sql(" WHERE (");
    self.table.all_columns().walk_ast(out.reborrow())?;
    self.push_sql(") = (");
    self.empty_row.walk_ast(out.reborrow())?;
    self.push_sql("))");
  }
}
