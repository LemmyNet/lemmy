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
use std::any::TypeId;
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
    let table = Q::Table::default;
    let deletion_condition = AllNull(
      Q::Table::all_columns()
        .into_array()
        .into_iter()
        .filter(|c: &DynColumn| {
          table()
            .primary_key()
            .into_array()
            .into_iter()
            .chain(self.set_null_columns.iter().cloned())
            .all(|excluded_column| excluded_column.type_id != c.type_id)
        })
        .collect::<Vec<_>>(),
    );
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
          .filter(dsl::not(deletion_condition.clone())),
      ),
      delete_subquery: Box::new(self.query.filter(deletion_condition)),
      table: Box::new(table()),
      primary_key: Box::new(table().primary_key()),
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

    // This is checked by require_uplete triggers
    out.push_sql("/**/");

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
      out.push_identifier(column.name)?;
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
    out.push_sql(") = ANY (SELECT * FROM delete_keys) RETURNING 1)");

    // Count updated rows and deleted rows (`RETURNING 1` makes this possible)
    out.push_sql(" SELECT (SELECT count(*) FROM update_result)");
    out.push_sql(", (SELECT count(*) FROM delete_result)");

    Ok(())
  }
}

// Types other than `DynColumn` are only used in tests
#[derive(Clone)]
pub struct AllNull<T = DynColumn>(Vec<T>);

impl<T> Expression for AllNull<T> {
  type SqlType = sql_types::Bool;
}

impl<T> ValidGrouping<()> for AllNull<T> {
  type IsAggregate = is_aggregate::No;
}

impl<T: QueryFragment<Pg>> QueryFragment<Pg> for AllNull<T> {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    // Must produce a valid expression even if `self.0` is empty
    out.push_sql("(TRUE");
    for item in &self.0 {
      out.push_sql(" AND (");
      item.walk_ast(out.reborrow())?;
      out.push_sql(" IS NULL)");
    }
    out.push_sql(")");

    Ok(())
  }
}

#[derive(Clone)]
pub struct DynColumn {
  type_id: TypeId,
  name: &'static str,
}

impl<T: Column + 'static> From<T> for DynColumn {
  fn from(_value: T) -> Self {
    DynColumn {
      type_id: TypeId::of::<T>(),
      name: T::NAME,
    }
  }
}

impl QueryFragment<Pg> for DynColumn {
  fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
    out.push_identifier(self.name)
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

#[cfg(test)]
mod tests {
  use super::AllNull;
  use crate::utils::{build_db_pool_for_tests, get_conn, DbConn};
  use diesel::{
    debug_query,
    insert_into,
    pg::Pg,
    query_builder::{AsQuery, QueryId},
    select,
    sql_types,
    AppearsOnTable,
    ExpressionMethods,
    IntoSql,
    QueryDsl,
    SelectableExpression,
  };
  use diesel_async::{RunQueryDsl, SimpleAsyncConnection};
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  impl<T, QS> AppearsOnTable<QS> for AllNull<T> {}

  impl<T, QS> SelectableExpression<QS> for AllNull<T> {}

  impl<T> QueryId for AllNull<T> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
  }

  diesel::table! {
    t (id1, id2) {
      // uplete doesn't work for non-tuple primary key
      id1 -> Int4,
      id2 -> Int4,
      a -> Nullable<Int4>,
      b -> Nullable<Int4>,
    }
  }

  async fn expect_rows(
    conn: &mut DbConn<'_>,
    expected: &[(Option<i32>, Option<i32>)],
  ) -> LemmyResult<()> {
    let rows: Vec<(Option<i32>, Option<i32>)> = t::table
      .select((t::a, t::b))
      .order_by(t::id1)
      .load(conn)
      .await?;
    assert_eq!(expected, &rows);

    Ok(())
  }

  // Main purpose of this test is to check accuracy of the returned `Count`, which other modules'
  // tests rely on
  #[tokio::test]
  #[serial]
  async fn test_count() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut conn = get_conn(pool).await?;

    conn
      .batch_execute("CREATE TABLE t (id1 serial, id2 int NOT NULL DEFAULT 1, a int, b int, PRIMARY KEY (id1, id2));")
      .await?;
    expect_rows(&mut conn, &[]).await?;

    insert_into(t::table)
      .values(&[
        (t::a.eq(Some(1)), t::b.eq(Some(2))),
        (t::a.eq(Some(3)), t::b.eq(None)),
        (t::a.eq(Some(4)), t::b.eq(Some(5))),
      ])
      .execute(&mut conn)
      .await?;
    expect_rows(
      &mut conn,
      &[(Some(1), Some(2)), (Some(3), None), (Some(4), Some(5))],
    )
    .await?;

    let count1 = super::new(t::table)
      .set_null(t::a)
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      super::Count {
        updated: 2,
        deleted: 1
      },
      count1
    );
    expect_rows(&mut conn, &[(None, Some(2)), (None, Some(5))]).await?;

    let count2 = super::new(t::table)
      .set_null(t::b)
      .get_result(&mut conn)
      .await?;
    assert_eq!(super::Count::only_deleted(2), count2);
    expect_rows(&mut conn, &[]).await?;

    conn.batch_execute("DROP TABLE t;").await?;

    Ok(())
  }

  fn expected_sql(check_null: &str, set_null: &str) -> String {
    let with_queries = {
      let key = r#""t"."id1", "t"."id2""#;
      let t = r#""t""#;

      let update_keys = format!("SELECT {key} FROM {t} WHERE  NOT (({check_null})) FOR UPDATE");
      let delete_keys = format!("SELECT {key} FROM {t} WHERE ({check_null}) FOR UPDATE");
      let update_result = format!(
        "UPDATE {t} SET {set_null} WHERE ({key}) = ANY (SELECT * FROM update_keys) RETURNING 1"
      );
      let delete_result =
        format!("DELETE FROM {t} WHERE ({key}) = ANY (SELECT * FROM delete_keys) RETURNING 1");

      format!("update_keys AS ({update_keys}), delete_keys AS ({delete_keys}), update_result AS ({update_result}), delete_result AS ({delete_result})")
    };
    let update_count = "SELECT count(*) FROM update_result";
    let delete_count = "SELECT count(*) FROM delete_result";

    format!(r#"/**/WITH {with_queries} SELECT ({update_count}), ({delete_count}) -- binds: []"#)
  }

  #[test]
  fn test_generated_sql() {
    // Unlike the `get_result` method, `debug_query` does not automatically call `as_query`
    assert_eq!(
      debug_query::<Pg, _>(&super::new(t::table).set_null(t::b).as_query()).to_string(),
      expected_sql(r#"TRUE AND ("a" IS NULL)"#, r#""b" = NULL"#)
    );
    assert_eq!(
      debug_query::<Pg, _>(
        &super::new(t::table)
          .set_null(t::a)
          .set_null(t::b)
          .as_query()
      )
      .to_string(),
      expected_sql(r#"TRUE"#, r#""a" = NULL,"b" = NULL"#)
    );
  }

  #[test]
  fn test_count_methods() {
    assert_eq!(
      super::Count::only_updated(1),
      super::Count {
        updated: 1,
        deleted: 0
      }
    );
    assert_eq!(
      super::Count::only_deleted(1),
      super::Count {
        updated: 0,
        deleted: 1
      }
    );
  }

  #[tokio::test]
  #[serial]
  async fn test_all_null() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut conn = get_conn(pool).await?;

    let some = Some(1).into_sql::<sql_types::Nullable<sql_types::Integer>>();
    let none = None::<i32>.into_sql::<sql_types::Nullable<sql_types::Integer>>();

    // Allows type inference for `vec![]`
    let mut all_null = |items| select(AllNull(items)).get_result::<bool>(&mut conn);

    assert!(all_null(vec![]).await?);
    assert!(all_null(vec![none]).await?);
    assert!(all_null(vec![none, none]).await?);
    assert!(all_null(vec![none, none, none]).await?);
    assert!(!all_null(vec![some]).await?);
    assert!(!all_null(vec![some, none]).await?);
    assert!(!all_null(vec![none, some, none]).await?);

    Ok(())
  }
}
