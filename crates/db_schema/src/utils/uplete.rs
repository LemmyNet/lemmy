use diesel::{
  pg::Pg,
  query_builder::{AstPass, QueryFragment},
  QueryResult, Table,
};

// TODO: use trait bounds to validate fields

/// Find each row in `target` that has its primary key listed in `keys`, and
/// if `delete_condition` is true, then delete the row, otherwise update it with `update_values`
#[derive(QueryId)]
pub struct Uplete<T, K, D, U> {
  pub target: T,
  pub keys: K,
  pub delete_condition: D,
  pub update_values: U,
}

impl<T: QueryFragment<Pg> + Table, K, D: QueryFragment<Pg>, U: QueryFragment<Pg>>
  QueryFragment<Pg> for Uplete<T, K, D, U>
  where
  for<'a> &'a K: IntoIterator,
  for<'a> <&'a K as IntoIterator>::Item: QueryFragment<Pg>,
  T::PrimaryKey: QueryFragment<Pg>
{
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
    // Needed because of the keys loop
    out.unsafe_to_cache_prepared();

    out.push_sql("MERGE INTO ");
    self.target.walk_ast(out.reborrow())?;
    out.push_sql("USING (VALUES ");
    for (i, key) in (&self.keys).into_iter().enumerate() {
      if i != 0 {
        out.push_sql(",");
      }
      out.push_sql("(ROW(");
      key.walk_ast(out.reborrow())?;
      out.push_sql("))");
    }
    out.push_sql(") AS uplete_source (uplete_key) ON ROW(");
    self.target.primary_key().walk_ast(out.reborrow())?;
    out.push_sql(") = uplete_source.uplete_key WHEN MATCHED AND (");
    self.delete_condition.walk_ast(out.reborrow())?;
    out.push_sql(") THEN DELETE WHEN MATCHED THEN UPDATE SET ");
    self.update_values.walk_ast(out.reborrow())?;

    Ok(())
  }
}
