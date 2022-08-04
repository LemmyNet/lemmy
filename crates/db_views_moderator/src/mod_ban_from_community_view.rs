use crate::structs::ModBanFromCommunityView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, mod_ban_from_community, person, person_alias_1},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModBanFromCommunity,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModBanFromCommunityViewTuple = (
  ModBanFromCommunity,
  PersonSafe,
  CommunitySafe,
  PersonSafeAlias1,
);

impl ModBanFromCommunityView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_ban_from_community::table
      .inner_join(person::table.on(mod_ban_from_community::mod_person_id.eq(person::id)))
      .inner_join(community::table)
      .inner_join(
        person_alias_1::table.on(mod_ban_from_community::other_person_id.eq(person_alias_1::id)),
      )
      .select((
        mod_ban_from_community::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_ban_from_community::mod_person_id.eq(mod_person_id));
    };

    if let Some(community_id) = community_id {
      query = query.filter(mod_ban_from_community::community_id.eq(community_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_ban_from_community::when_.desc())
      .load::<ModBanFromCommunityViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModBanFromCommunityView {
  type DbTuple = ModBanFromCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_ban_from_community: a.0,
        moderator: a.1,
        community: a.2,
        banned_person: a.3,
      })
      .collect::<Vec<Self>>()
  }
}
