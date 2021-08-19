use diesel::{result::Error, *};
use lemmy_db_queries::{
  aggregates::comment_aggregates::CommentAggregates,
  functions::hot_rank,
  limit_and_offset,
  MaybeOptional,
  SortType,
  ToSafe,
  ViewToVec,
};
use lemmy_db_schema::{
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_person_ban,
    person,
    person_alias_1,
    person_block,
    person_mention,
    post,
  },
  source::{
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
    person_block::PersonBlock,
    person_mention::PersonMention,
    post::Post,
  },
  PersonId,
  PersonMentionId,
};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PersonMentionView {
  pub person_mention: PersonMention,
  pub comment: Comment,
  pub creator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
  pub recipient: PersonSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub subscribed: bool,                    // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub creator_blocked: bool,               // Left join to PersonBlock
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

type PersonMentionViewTuple = (
  PersonMention,
  Comment,
  PersonSafe,
  Post,
  CommunitySafe,
  PersonSafeAlias1,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl PersonMentionView {
  pub fn read(
    conn: &PgConnection,
    person_mention_id: PersonMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (
      person_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      creator_blocked,
      my_vote,
    ) = person_mention::table
      .find(person_mention_id)
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        person_mention::all_columns,
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<PersonMentionViewTuple>(conn)?;

    Ok(PersonMentionView {
      person_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: subscribed.is_some(),
      saved: saved.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
    })
  }
}

pub struct PersonMentionQueryBuilder<'a> {
  conn: &'a PgConnection,
  my_person_id: Option<PersonId>,
  recipient_id: Option<PersonId>,
  sort: Option<SortType>,
  unread_only: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PersonMentionQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    PersonMentionQueryBuilder {
      conn,
      my_person_id: None,
      recipient_id: None,
      sort: None,
      unread_only: None,
      page: None,
      limit: None,
    }
  }

  pub fn sort<T: MaybeOptional<SortType>>(mut self, sort: T) -> Self {
    self.sort = sort.get_optional();
    self
  }

  pub fn unread_only<T: MaybeOptional<bool>>(mut self, unread_only: T) -> Self {
    self.unread_only = unread_only.get_optional();
    self
  }

  pub fn recipient_id<T: MaybeOptional<PersonId>>(mut self, recipient_id: T) -> Self {
    self.recipient_id = recipient_id.get_optional();
    self
  }

  pub fn my_person_id<T: MaybeOptional<PersonId>>(mut self, my_person_id: T) -> Self {
    self.my_person_id = my_person_id.get_optional();
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<PersonMentionView>, Error> {
    use diesel::dsl::*;

    // The left join below will return None in this case
    let person_id_join = self.my_person_id.unwrap_or(PersonId(-1));

    let mut query = person_mention::table
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        person_mention::all_columns,
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    if let Some(recipient_id) = self.recipient_id {
      query = query.filter(person_mention::recipient_id.eq(recipient_id));
    }

    if self.unread_only.unwrap_or(false) {
      query = query.filter(person_mention::read.eq(false));
    }

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::Hot | SortType::Active => query
        .order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      SortType::New | SortType::MostComments | SortType::NewComments => {
        query.order_by(comment::published.desc())
      }
      SortType::TopAll => query.order_by(comment_aggregates::score.desc()),
      SortType::TopYear => query
        .filter(comment::published.gt(now - 1.years()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopMonth => query
        .filter(comment::published.gt(now - 1.months()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopWeek => query
        .filter(comment::published.gt(now - 1.weeks()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopDay => query
        .filter(comment::published.gt(now - 1.days()))
        .order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<PersonMentionViewTuple>(self.conn)?;

    Ok(PersonMentionView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PersonMentionView {
  type DbTuple = PersonMentionViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        person_mention: a.0.to_owned(),
        comment: a.1.to_owned(),
        creator: a.2.to_owned(),
        post: a.3.to_owned(),
        community: a.4.to_owned(),
        recipient: a.5.to_owned(),
        counts: a.6.to_owned(),
        creator_banned_from_community: a.7.is_some(),
        subscribed: a.8.is_some(),
        saved: a.9.is_some(),
        creator_blocked: a.10.is_some(),
        my_vote: a.11,
      })
      .collect::<Vec<Self>>()
  }
}
