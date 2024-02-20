use diesel::{
  pg::Pg,
  query_builder::{AstPass, QueryFragment},
  QueryResult,
};

// TODO: use trait bounds to validate fields

#[derive(QueryId)]
pub struct Uplete<T, F, D, U> {
  pub target: T,
  /// Must only match 1 row
  pub filter: F,
  pub delete_condition: D,
  pub update_values: U,
}

impl<T: QueryFragment<Pg>, F: QueryFragment<Pg>, D: QueryFragment<Pg>, U: QueryFragment<Pg>> QueryFragment<Pg>
  for Uplete<T, F, D, U>
{
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
    out.push_sql("MERGE INTO ");
    self.target.walk_ast(out.reborrow())?;
    out.push_sql("USING (VALUES (1)) AS uplete_source ON (");
    self.filter.walk_ast(out.reborrow())?;
    out.push_sql(") WHEN MATCHED AND (");
    self.delete_condition.walk_ast(out.reborrow())?;
    out.push_sql(") THEN DELETE WHEN MATCHED THEN UPDATE SET ");
    self.update_values.walk_ast(out.reborrow())?;

    Ok(())
  }
}
