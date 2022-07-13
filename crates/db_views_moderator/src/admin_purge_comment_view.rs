use crate::structs::AdminPurgeCommentView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_comment, person, post},
  source::{
    moderator::AdminPurgeComment,
    person::{Person, PersonSafe},
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type AdminPurgeCommentViewTuple = (AdminPurgeComment, PersonSafe, Post);

impl AdminPurgeCommentView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = admin_purge_comment::table
      .inner_join(person::table.on(admin_purge_comment::admin_person_id.eq(person::id)))
      .inner_join(post::table)
      .select((
        admin_purge_comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_comment::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_comment::when_.desc())
      .load::<AdminPurgeCommentViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for AdminPurgeCommentView {
  type DbTuple = AdminPurgeCommentViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        admin_purge_comment: a.0,
        admin: a.1,
        post: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
