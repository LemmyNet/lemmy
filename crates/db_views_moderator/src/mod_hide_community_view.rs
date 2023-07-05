use crate::structs::{ModHideCommunityView, ModlogListParams};
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
  schema::{community, mod_hide_community, person},
  source::{community::Community, moderator::ModHideCommunity, person::Person},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};

type ModHideCommunityViewTuple = (ModHideCommunity, Option<Person>, Community);

impl ModHideCommunityView {
  // Pass in mod_id as admin_id because only admins can do this action
  pub async fn list(pool: &DbPool, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_hide_community::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_hide_community::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(community::table.on(mod_hide_community::community_id.eq(community::id)))
      .select((
        mod_hide_community::all_columns,
        person::all_columns.nullable(),
        community::all_columns,
      ))
      .into_boxed();

    if let Some(community_id) = params.community_id {
      query = query.filter(mod_hide_community::community_id.eq(community_id));
    };

    if let Some(admin_id) = params.mod_person_id {
      query = query.filter(mod_hide_community::mod_person_id.eq(admin_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_hide_community::when_.desc())
      .load::<ModHideCommunityViewTuple>(conn)
      .await?;

    let results = res.into_iter().map(Self::from_tuple).collect();
    Ok(results)
  }
}

impl JoinView for ModHideCommunityView {
  type JoinTuple = ModHideCommunityViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      mod_hide_community: a.0,
      admin: a.1,
      community: a.2,
    }
  }
}
