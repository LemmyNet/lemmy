use crate::structs::ModStickyPostView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, mod_sticky_post, person, person_alias_1, post},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModStickyPost,
    person::{Person, PersonSafe},
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModStickyPostViewTuple = (ModStickyPost, Option<PersonSafe>, Post, CommunitySafe);

impl ModStickyPostView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    other_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !hide_mod_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_sticky_post::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_sticky_post::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(post::table)
      .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_sticky_post::all_columns,
        Person::safe_columns_tuple().nullable(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_sticky_post::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_sticky_post::when_.desc())
      .load::<ModStickyPostViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for ModStickyPostView {
  type DbTuple = ModStickyPostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_sticky_post: a.0,
        moderator: a.1,
        post: a.2,
        community: a.3,
      })
      .collect::<Vec<Self>>()
  }
}
