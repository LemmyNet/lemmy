use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{comment, community, mod_remove_comment, person, person_alias_1, post},
  source::{
    comment::Comment,
    community::{Community, CommunitySafe},
    moderator::ModRemoveComment,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
    post::Post,
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: PersonSafe,
  pub comment: Comment,
  pub commenter: PersonSafeAlias1,
  pub post: Post,
  pub community: CommunitySafe,
}

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

    let (limit, offset) = limit_and_offset(page, limit);

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
      .iter()
      .map(|a| Self {
        mod_remove_comment: a.0.to_owned(),
        moderator: a.1.to_owned(),
        comment: a.2.to_owned(),
        commenter: a.3.to_owned(),
        post: a.4.to_owned(),
        community: a.5.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
