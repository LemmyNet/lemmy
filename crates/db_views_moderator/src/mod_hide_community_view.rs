use crate::structs::ModHideCommunityView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, mod_hide_community, person},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModHideCommunity,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModHideCommunityViewTuple = (ModHideCommunity, PersonSafe, CommunitySafe);

impl ModHideCommunityView {
  // Pass in mod_id as admin_id because only admins can do this action
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    admin_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_hide_community::table
      .inner_join(person::table)
      .inner_join(community::table.on(mod_hide_community::community_id.eq(community::id)))
      .select((
        mod_hide_community::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(mod_hide_community::community_id.eq(community_id));
    };

    if let Some(admin_id) = admin_id {
      query = query.filter(mod_hide_community::mod_person_id.eq(admin_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_hide_community::when_.desc())
      .load::<ModHideCommunityViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModHideCommunityView {
  type DbTuple = ModHideCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_hide_community: a.0,
        admin: a.1,
        community: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
