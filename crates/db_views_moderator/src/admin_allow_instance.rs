use crate::structs::{AdminAllowInstanceView, ModlogListParams};
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
  schema::{admin_allow_instance, instance, person},
  utils::{get_conn, limit_and_offset, DbPool},
};

impl AdminAllowInstanceView {
  pub async fn list(pool: &mut DbPool<'_>, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = admin_allow_instance::admin_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = admin_allow_instance::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(instance::table)
      .select((
        admin_allow_instance::all_columns,
        instance::all_columns,
        person::all_columns.nullable(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = params.mod_person_id {
      query = query.filter(admin_allow_instance::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .order_by(admin_allow_instance::when_.desc())
      .load::<Self>(conn)
      .await
  }
}
