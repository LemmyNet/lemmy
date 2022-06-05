use crate::structs::AdminPurgePersonView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_person, person},
  source::{
    moderator::AdminPurgePerson,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type AdminPurgePersonViewTuple = (AdminPurgePerson, Option<PersonSafe>);

impl AdminPurgePersonView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = admin_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !hide_mod_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = admin_purge_person::admin_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = admin_purge_person::table
      .left_join(person::table.on(admin_names_join))
      .select((
        admin_purge_person::all_columns,
        Person::safe_columns_tuple().nullable(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_person::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_person::when_.desc())
      .load::<AdminPurgePersonViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for AdminPurgePersonView {
  type DbTuple = AdminPurgePersonViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        admin_purge_person: a.0,
        admin: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
