use crate::structs::{ModAddCommunityView, ModlogListParams};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, mod_add_community, person, person_alias_1},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModAddCommunity,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModAddCommunityViewTuple = (
  ModAddCommunity,
  Option<PersonSafe>,
  CommunitySafe,
  PersonSafeAlias1,
);

impl ModAddCommunityView {
  pub fn list(conn: &mut PgConnection, params: ModlogListParams) -> Result<Vec<Self>, Error> {
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
        person_alias_1::table.on(mod_add_community::other_person_id.eq(person_alias_1::id)),
      )
      .select((
        mod_add_community::all_columns,
        Person::safe_columns_tuple().nullable(),
        Community::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_add_community::mod_person_id.eq(mod_person_id));
    };

    if let Some(community_id) = params.community_id {
      query = query.filter(mod_add_community::community_id.eq(community_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add_community::when_.desc())
      .load::<ModAddCommunityViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for ModAddCommunityView {
  type DbTuple = ModAddCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_add_community: a.0,
        moderator: a.1,
        community: a.2,
        modded_person: a.3,
      })
      .collect::<Vec<Self>>()
  }
}
