use crate::structs::{ModBanView, ModlogListParams};
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
  schema::{mod_ban, person},
  source::{moderator::ModBan, person::Person},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};

type ModBanViewTuple = (ModBan, Option<Person>, Person);

impl ModBanView {
  pub async fn list(pool: &DbPool, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_ban::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_ban::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(person_alias_1.on(mod_ban::other_person_id.eq(person_alias_1.field(person::id))))
      .select((
        mod_ban::all_columns,
        person::all_columns.nullable(),
        person_alias_1.fields(person::all_columns),
      ))
      .into_boxed();

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_ban::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1.field(person::id).eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_ban::when_.desc())
      .load::<ModBanViewTuple>(conn)
      .await?;

    let results = res.into_iter().map(Self::from_tuple).collect();
    Ok(results)
  }
}

impl JoinView for ModBanView {
  type JoinTuple = ModBanViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      mod_ban: a.0,
      moderator: a.1,
      banned_person: a.2,
    }
  }
}
