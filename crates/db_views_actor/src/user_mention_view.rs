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
    community_user_ban,
    post,
    user_,
    user_alias_1,
    user_mention,
  },
  source::{
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunitySafe, CommunityUserBan},
    post::Post,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
    user_mention::UserMention,
  },
};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct UserMentionView {
  pub user_mention: UserMention,
  pub comment: Comment,
  pub creator: UserSafe,
  pub post: Post,
  pub community: CommunitySafe,
  pub recipient: UserSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityUserBan
  pub subscribed: bool,                    // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

type UserMentionViewTuple = (
  UserMention,
  Comment,
  UserSafe,
  Post,
  CommunitySafe,
  UserSafeAlias1,
  CommentAggregates,
  Option<CommunityUserBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<i16>,
);

impl UserMentionView {
  pub fn read(
    conn: &PgConnection,
    user_mention_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (
      user_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      my_vote,
    ) = user_mention::table
      .find(user_mention_id)
      .inner_join(comment::table)
      .inner_join(user_::table.on(comment::creator_id.eq(user_::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(user_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_user_ban::table.on(
          community::id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        user_mention::all_columns,
        comment::all_columns,
        User_::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<UserMentionViewTuple>(conn)?;

    Ok(UserMentionView {
      user_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: subscribed.is_some(),
      saved: saved.is_some(),
      my_vote,
    })
  }
}

pub struct UserMentionQueryBuilder<'a> {
  conn: &'a PgConnection,
  my_user_id: Option<i32>,
  recipient_id: Option<i32>,
  sort: &'a SortType,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserMentionQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    UserMentionQueryBuilder {
      conn,
      my_user_id: None,
      recipient_id: None,
      sort: &SortType::New,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn unread_only(mut self, unread_only: bool) -> Self {
    self.unread_only = unread_only;
    self
  }

  pub fn recipient_id<T: MaybeOptional<i32>>(mut self, recipient_id: T) -> Self {
    self.recipient_id = recipient_id.get_optional();
    self
  }

  pub fn my_user_id<T: MaybeOptional<i32>>(mut self, my_user_id: T) -> Self {
    self.my_user_id = my_user_id.get_optional();
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

  pub fn list(self) -> Result<Vec<UserMentionView>, Error> {
    use diesel::dsl::*;

    // The left join below will return None in this case
    let user_id_join = self.my_user_id.unwrap_or(-1);

    let mut query = user_mention::table
      .inner_join(comment::table)
      .inner_join(user_::table.on(comment::creator_id.eq(user_::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(user_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_user_ban::table.on(
          community::id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        user_mention::all_columns,
        comment::all_columns,
        User_::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    if let Some(recipient_id) = self.recipient_id {
      query = query.filter(user_mention::recipient_id.eq(recipient_id));
    }

    if self.unread_only {
      query = query.filter(user_mention::read.eq(false));
    }

    query = match self.sort {
      SortType::Hot | SortType::Active => query
        .order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      SortType::New | SortType::MostComments => query.order_by(comment::published.desc()),
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
      .load::<UserMentionViewTuple>(self.conn)?;

    Ok(UserMentionView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for UserMentionView {
  type DbTuple = UserMentionViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        user_mention: a.0.to_owned(),
        comment: a.1.to_owned(),
        creator: a.2.to_owned(),
        post: a.3.to_owned(),
        community: a.4.to_owned(),
        recipient: a.5.to_owned(),
        counts: a.6.to_owned(),
        creator_banned_from_community: a.7.is_some(),
        subscribed: a.8.is_some(),
        saved: a.9.is_some(),
        my_vote: a.10,
      })
      .collect::<Vec<Self>>()
  }
}
