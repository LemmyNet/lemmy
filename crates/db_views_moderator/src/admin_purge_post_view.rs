use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{admin_purge_post, community, person},
  source::{
    community::Community,
    moderator::AdminPurgePost,
    person::{Person, PersonSafe},
  },
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: PersonSafe,
  pub community: Community,
}

type AdminPurgePostViewTuple = (AdminPurgePost, PersonSafe, Community);

impl AdminPurgePostView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = admin_purge_post::table
      .inner_join(person::table.on(admin_purge_post::admin_person_id.eq(person::id)))
      .inner_join(community::table)
      .select((
        admin_purge_post::all_columns,
        Person::safe_columns_tuple(),
        community::all_columns,
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_post::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_post::when_.desc())
      .load::<AdminPurgePostViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for AdminPurgePostView {
  type DbTuple = AdminPurgePostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        admin_purge_post: a.0.to_owned(),
        admin: a.1.to_owned(),
        community: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
