use crate::structs::ModAddView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{mod_add, person, person_alias_1},
  source::{
    moderator::ModAdd,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModAddViewTuple = (ModAdd, Option<PersonSafe>, PersonSafeAlias1);

impl ModAddView {
  pub fn list(
    conn: &PgConnection,
    mod_person_id: Option<PersonId>,
    other_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !hide_mod_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_add::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_add::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(person_alias_1::table.on(mod_add::other_person_id.eq(person_alias_1::id)))
      .select((
        mod_add::all_columns,
        Person::safe_columns_tuple().nullable(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_add::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add::when_.desc())
      .load::<ModAddViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for ModAddView {
  type DbTuple = ModAddViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_add: a.0,
        moderator: a.1,
        modded_person: a.2,
      })
      .collect::<Vec<Self>>()
  }
}
