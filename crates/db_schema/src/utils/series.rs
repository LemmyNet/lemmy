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
  SelectableExpression,
};

/// Gererates a series of rows for insertion.
///
/// An inclusive range is created from `start` and `stop`. A row for each number is generated using `selection`, which can be a tuple.
/// [`current_value`] is an expression that gets the current value.
///
/// For example, if there's a `numbers` table with a `number` column, this inserts all numbers from 1 to 10 in a single statement:
///
/// ```
/// dsl::insert_into(numbers::table)
///   .values(ValuesFromSeries {
///     start: 1,
///     stop: 10,
///     selection: series::current_value,
///   })
///   .into_columns(numbers::number)
/// ```
#[derive(QueryId)]
pub struct ValuesFromSeries<S> {
  pub start: i64,
  pub stop: i64,
  pub selection: S,
}

impl<S: QueryFragment<Pg>> QueryFragment<Pg> for ValuesFromSeries<S> {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    self.selection.walk_ast(out.reborrow())?;
    out.push_sql(" FROM generate_series(");
    out.push_bind_param::<sql_types::BigInt, _>(&self.start)?;
    out.push_sql(", ");
    out.push_bind_param::<sql_types::BigInt, _>(&self.stop)?;
    out.push_sql(")");

    Ok(())
  }
}

impl<S: Expression> Expression for ValuesFromSeries<S> {
  type SqlType = S::SqlType;
}

impl<T, S: AppearsOnTable<current_value>> AppearsOnTable<T> for ValuesFromSeries<S> {}

impl<T, S: SelectableExpression<current_value>> SelectableExpression<T> for ValuesFromSeries<S> {}

impl<T, S: SelectableExpression<current_value>> Insertable<T> for ValuesFromSeries<S>
where
  dsl::BareSelect<Self>: AsQuery + Insertable<T>,
{
  type Values = <dsl::BareSelect<Self> as Insertable<T>>::Values;

  fn values(self) -> Self::Values {
    dsl::select(self).values()
  }
}

impl<S: ValidGrouping<(), IsAggregate = is_aggregate::No>> ValidGrouping<()>
  for ValuesFromSeries<S>
{
  type IsAggregate = is_aggregate::No;
}

#[allow(non_camel_case_types)]
#[derive(QueryId, Clone, Copy, Debug)]
pub struct current_value;

impl QueryFragment<Pg> for current_value {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    out.push_identifier("generate_series")?;

    Ok(())
  }
}

impl Expression for current_value {
  type SqlType = sql_types::BigInt;
}

impl AppearsOnTable<current_value> for current_value {}

impl SelectableExpression<current_value> for current_value {}

impl ValidGrouping<()> for current_value {
  type IsAggregate = is_aggregate::No;
}
