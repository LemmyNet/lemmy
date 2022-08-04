use crate::structs::AdminPurgePostView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_post, community, person},
  source::{
    community::{Community, CommunitySafe},
    moderator::AdminPurgePost,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type AdminPurgePostViewTuple = (AdminPurgePost, PersonSafe, CommunitySafe);

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
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_post::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

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
      .into_iter()
      .map(|a| Self {
        admin_purge_post: a.0,
        admin: a.1,
        community: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
