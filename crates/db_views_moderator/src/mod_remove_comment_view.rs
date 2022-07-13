use crate::structs::ModRemoveCommentView;
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
  PersonSafe,
  Comment,
  PersonSafeAlias1,
  Post,
  CommunitySafe,
);

impl ModRemoveCommentView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_remove_comment::table
      .inner_join(person::table)
      .inner_join(comment::table)
      .inner_join(person_alias_1::table.on(comment::creator_id.eq(person_alias_1::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_remove_comment::all_columns,
        Person::safe_columns_tuple(),
        comment::all_columns,
        PersonAlias1::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_remove_comment::mod_person_id.eq(mod_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_comment::when_.desc())
      .load::<ModRemoveCommentViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
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
