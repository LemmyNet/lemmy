use crate::structs::ModBanView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{mod_ban, person, person_alias_1},
  source::{
    moderator::ModBan,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModBanViewTuple = (ModBan, PersonSafe, PersonSafeAlias1);

impl ModBanView {
  pub fn list(
    conn: &PgConnection,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_ban::table
      .inner_join(person::table.on(mod_ban::mod_person_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(mod_ban::other_person_id.eq(person_alias_1::id)))
      .select((
        mod_ban::all_columns,
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_ban::mod_person_id.eq(mod_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_ban::when_.desc())
      .load::<ModBanViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModBanView {
  type DbTuple = ModBanViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_ban: a.0,
        moderator: a.1,
        banned_person: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
