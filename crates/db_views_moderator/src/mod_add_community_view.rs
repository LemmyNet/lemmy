use crate::structs::{ModAddCommunityView, ModlogListParams};
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
  schema::{community, mod_add_community, person},
  source::{community::Community, moderator::ModAddCommunity, person::Person},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};

type ModAddCommunityViewTuple = (ModAddCommunity, Option<Person>, Community, Person);

impl ModAddCommunityView {
  pub async fn list(pool: &DbPool, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_add_community::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_add_community::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(community::table)
      .inner_join(
        person_alias_1.on(mod_add_community::other_person_id.eq(person_alias_1.field(person::id))),
      )
      .select((
        mod_add_community::all_columns,
        person::all_columns.nullable(),
        community::all_columns,
        person_alias_1.fields(person::all_columns),
      ))
      .into_boxed();

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_add_community::mod_person_id.eq(mod_person_id));
    };

    if let Some(community_id) = params.community_id {
      query = query.filter(mod_add_community::community_id.eq(community_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1.field(person::id).eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add_community::when_.desc())
      .load::<ModAddCommunityViewTuple>(conn)
      .await?;

    let results = res.into_iter().map(Self::from_tuple).collect();
    Ok(results)
  }
}

impl JoinView for ModAddCommunityView {
  type JoinTuple = ModAddCommunityViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      mod_add_community: a.0,
      moderator: a.1,
      community: a.2,
      modded_person: a.3,
    }
  }
}
