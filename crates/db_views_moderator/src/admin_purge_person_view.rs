use crate::structs::{AdminPurgePersonView, ModlogListParams};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_person, person},
  source::{moderator::AdminPurgePerson, person::Person},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};

type AdminPurgePersonViewTuple = (AdminPurgePerson, Option<Person>);

impl AdminPurgePersonView {
  pub async fn list(pool: &DbPool, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = admin_purge_person::admin_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = admin_purge_person::table
      .left_join(person::table.on(admin_names_join))
      .select((
        admin_purge_person::all_columns,
        person::all_columns.nullable(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = params.mod_person_id {
      query = query.filter(admin_purge_person::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_person::when_.desc())
      .load::<AdminPurgePersonViewTuple>(conn)
      .await?;

    let results = res.into_iter().map(Self::from_tuple).collect();
    Ok(results)
  }
}

impl JoinView for AdminPurgePersonView {
  type JoinTuple = AdminPurgePersonViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      admin_purge_person: a.0,
      admin: a.1,
    }
  }
}
