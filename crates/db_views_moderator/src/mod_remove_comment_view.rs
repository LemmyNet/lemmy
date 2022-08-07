use crate::structs::{ModRemoveCommentView, ModlogListParams};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{comment, community, mod_remove_comment, person, person_alias_1, post},
  source::{
    comment::Comment,
    community::{Community, CommunitySafe},
    moderator::ModRemoveComment,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModRemoveCommentViewTuple = (
  ModRemoveComment,
  Option<PersonSafe>,
  Comment,
  PersonSafeAlias1,
  Post,
  CommunitySafe,
);

impl ModRemoveCommentView {
  pub fn list(conn: &PgConnection, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_remove_comment::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_remove_comment::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(comment::table)
      .inner_join(person_alias_1::table.on(comment::creator_id.eq(person_alias_1::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_remove_comment::all_columns,
        Person::safe_columns_tuple().nullable(),
        comment::all_columns,
        PersonAlias1::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = params.community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_remove_comment::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_comment::when_.desc())
      .load::<ModRemoveCommentViewTuple>(conn)?;

    let results = Self::from_tuple_to_vec(res);
    Ok(results)
  }
}

impl ViewToVec for ModRemoveCommentView {
  type DbTuple = ModRemoveCommentViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        mod_remove_comment: a.0,
        moderator: a.1,
        comment: a.2,
        commenter: a.3,
        post: a.4,
        community: a.5,
      })
      .collect::<Vec<Self>>()
  }
}
