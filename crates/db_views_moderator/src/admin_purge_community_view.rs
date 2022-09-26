use crate::structs::{AdminPurgeCommunityView, ModlogListParams};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_community, person},
  source::{
    moderator::AdminPurgeCommunity,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type AdminPurgeCommunityViewTuple = (AdminPurgeCommunity, Option<PersonSafe>);

impl AdminPurgeCommunityView {
  pub fn list(conn: &mut PgConnection, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = admin_purge_community::admin_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));

    let mut query = admin_purge_community::table
      .left_join(person::table.on(admin_names_join))
      .select((
        admin_purge_community::all_columns,
        Person::safe_columns_tuple().nullable(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = params.mod_person_id {
      query = query.filter(admin_purge_community::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_community::when_.desc())
      .load::<AdminPurgeCommunityViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for AdminPurgeCommunityView {
  type DbTuple = AdminPurgeCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        admin_purge_community: a.0,
        admin: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
