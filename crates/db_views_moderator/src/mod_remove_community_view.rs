use crate::structs::{ModRemoveCommunityView, ModlogListParams};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, mod_remove_community, person},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModRemoveCommunity,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModRemoveCommunityTuple = (ModRemoveCommunity, Option<PersonSafe>, CommunitySafe);

impl ModRemoveCommunityView {
  pub fn list(conn: &mut PgConnection, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_remove_community::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_remove_community::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(community::table)
      .select((
        mod_remove_community::all_columns,
        Person::safe_columns_tuple().nullable(),
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_remove_community::mod_person_id.eq(mod_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_community::when_.desc())
      .load::<ModRemoveCommunityTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for ModRemoveCommunityView {
  type DbTuple = ModRemoveCommunityTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_remove_community: a.0,
        moderator: a.1,
        community: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
