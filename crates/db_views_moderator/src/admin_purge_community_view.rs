use crate::structs::AdminPurgeCommunityView;
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

type AdminPurgeCommunityViewTuple = (AdminPurgeCommunity, PersonSafe);

impl AdminPurgeCommunityView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = admin_purge_community::table
      .inner_join(person::table.on(admin_purge_community::admin_person_id.eq(person::id)))
      .select((
        admin_purge_community::all_columns,
        Person::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_community::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_community::when_.desc())
      .load::<AdminPurgeCommunityViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
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
