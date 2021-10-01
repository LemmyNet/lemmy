use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{admin_purge, person},
  source::{
    moderator::AdminPurge,
    person::{Person, PersonSafe},
  },
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AdminPurgeView {
  pub admin_purge: AdminPurge,
  pub admin: PersonSafe,
}

type AdminPurgeViewTuple = (AdminPurge, PersonSafe);

impl AdminPurgeView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = admin_purge::table
      .inner_join(person::table.on(admin_purge::admin_person_id.eq(person::id)))
      .select((admin_purge::all_columns, Person::safe_columns_tuple()))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge::when_.desc())
      .load::<AdminPurgeViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for AdminPurgeView {
  type DbTuple = AdminPurgeViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        admin_purge: a.0.to_owned(),
        admin: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
