use crate::structs::{ModRemovePostView, ModlogListParams};
use diesel::{
  result::Error, BoolExpressionMethods, ExpressionMethods, IntoSql, JoinOnDsl,
  NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, mod_remove_post, person, post},
  utils::{get_conn, limit_and_offset, DbPool},
};

impl ModRemovePostView {
  pub async fn list(pool: &mut DbPool<'_>, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let person_alias_1 = diesel::alias!(person as person1);
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_remove_post::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_remove_post::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1.on(post::creator_id.eq(person_alias_1.field(person::id))))
      .select((
        mod_remove_post::all_columns,
        person::all_columns.nullable(),
        post::all_columns,
        community::all_columns,
      ))
      .into_boxed();

    if let Some(community_id) = params.community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_remove_post::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1.field(person::id).eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_post::when_.desc())
      .load::<ModRemovePostView>(conn)
      .await
  }
}
