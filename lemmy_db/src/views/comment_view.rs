use crate::{
  aggregates::comment_aggregates::CommentAggregates,
  functions::hot_rank,
  fuzzy_search,
  limit_and_offset,
  schema::{
    comment,
    comment_aggregates,
    comment_alias_1,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_user_ban,
    post,
    user_,
    user_alias_1,
  },
  source::{
    comment::{Comment, CommentAlias1, CommentSaved},
    community::{Community, CommunityFollower, CommunitySafe, CommunityUserBan},
    post::Post,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
  views::ViewToVec,
  ListingType,
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct CommentView {
  pub comment: Comment,
  pub creator: UserSafe,
  pub recipient: Option<UserSafeAlias1>, // Left joins to comment and user
  pub post: Post,
  pub community: CommunitySafe,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityUserBan
  pub subscribed: bool,                    // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

type CommentViewTuple = (
  Comment,
  UserSafe,
  Option<CommentAlias1>,
  Option<UserSafeAlias1>,
  Post,
  CommunitySafe,
  CommentAggregates,
  Option<CommunityUserBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<i16>,
);

impl CommentView {
  pub fn read(
    conn: &PgConnection,
    comment_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (
      comment,
      creator,
      _parent_comment,
      recipient,
      post,
      community,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      my_vote,
    ) = comment::table
      .find(comment_id)
      .inner_join(user_::table)
      // recipient here
      .left_join(comment_alias_1::table.on(comment_alias_1::id.nullable().eq(comment::parent_id)))
      .left_join(user_alias_1::table.on(user_alias_1::id.eq(comment_alias_1::creator_id)))
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
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
        comment::all_columns,
        User_::safe_columns_tuple(),
        comment_alias_1::all_columns.nullable(),
        UserAlias1::safe_columns_tuple().nullable(),
        post::all_columns,
        Community::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<CommentViewTuple>(conn)?;

    Ok(CommentView {
      comment,
      recipient,
      post,
      creator,
      community,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: subscribed.is_some(),
      saved: saved.is_some(),
      my_vote,
    })
  }
}

mod join_types {
  use crate::schema::{
    comment,
    comment_aggregates,
    comment_alias_1,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_user_ban,
    post,
    user_,
    user_alias_1,
  };
  use diesel::{
    pg::Pg,
    query_builder::BoxedSelectStatement,
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::*,
  };

  // /// TODO awful, but necessary because of the boxed join
  pub(super) type BoxedCommentJoin<'a> = BoxedSelectStatement<
    'a,
    (
      (
        Integer,
        Integer,
        Integer,
        Nullable<Integer>,
        Text,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Text,
        Bool,
      ),
      (
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Bool,
        Nullable<Text>,
        Bool,
      ),
      Nullable<(
        Integer,
        Integer,
        Integer,
        Nullable<Integer>,
        Text,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Text,
        Bool,
      )>,
      Nullable<(
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Bool,
        Nullable<Text>,
        Bool,
      )>,
      (
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Integer,
        Integer,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Bool,
        Bool,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Text,
        Bool,
      ),
      (
        Integer,
        Text,
        Text,
        Nullable<Text>,
        Integer,
        Integer,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Bool,
        Text,
        Bool,
        Nullable<Text>,
        Nullable<Text>,
      ),
      (Integer, Integer, BigInt, BigInt, BigInt),
      Nullable<(Integer, Integer, Integer, Timestamp)>,
      Nullable<(Integer, Integer, Integer, Timestamp, Nullable<Bool>)>,
      Nullable<(Integer, Integer, Integer, Timestamp)>,
      Nullable<SmallInt>,
    ),
    JoinOn<
      Join<
        JoinOn<
          Join<
            JoinOn<
              Join<
                JoinOn<
                  Join<
                    JoinOn<
                      Join<
                        JoinOn<
                          Join<
                            JoinOn<
                              Join<
                                JoinOn<
                                  Join<
                                    JoinOn<
                                      Join<
                                        JoinOn<
                                          Join<comment::table, user_::table, Inner>,
                                          diesel::expression::operators::Eq<
                                            diesel::expression::nullable::Nullable<
                                              comment::columns::creator_id,
                                            >,
                                            diesel::expression::nullable::Nullable<
                                              user_::columns::id,
                                            >,
                                          >,
                                        >,
                                        comment_alias_1::table,
                                        LeftOuter,
                                      >,
                                      diesel::expression::operators::Eq<
                                        diesel::expression::nullable::Nullable<
                                          comment_alias_1::columns::id,
                                        >,
                                        comment::columns::parent_id,
                                      >,
                                    >,
                                    user_alias_1::table,
                                    LeftOuter,
                                  >,
                                  diesel::expression::operators::Eq<
                                    user_alias_1::columns::id,
                                    comment_alias_1::columns::creator_id,
                                  >,
                                >,
                                post::table,
                                Inner,
                              >,
                              diesel::expression::operators::Eq<
                                diesel::expression::nullable::Nullable<comment::columns::post_id>,
                                diesel::expression::nullable::Nullable<post::columns::id>,
                              >,
                            >,
                            community::table,
                            Inner,
                          >,
                          diesel::expression::operators::Eq<
                            post::columns::community_id,
                            community::columns::id,
                          >,
                        >,
                        comment_aggregates::table,
                        Inner,
                      >,
                      diesel::expression::operators::Eq<
                        diesel::expression::nullable::Nullable<
                          comment_aggregates::columns::comment_id,
                        >,
                        diesel::expression::nullable::Nullable<comment::columns::id>,
                      >,
                    >,
                    community_user_ban::table,
                    LeftOuter,
                  >,
                  diesel::expression::operators::And<
                    diesel::expression::operators::Eq<
                      community::columns::id,
                      community_user_ban::columns::community_id,
                    >,
                    diesel::expression::operators::Eq<
                      community_user_ban::columns::user_id,
                      comment::columns::creator_id,
                    >,
                  >,
                >,
                community_follower::table,
                LeftOuter,
              >,
              diesel::expression::operators::And<
                diesel::expression::operators::Eq<
                  post::columns::community_id,
                  community_follower::columns::community_id,
                >,
                diesel::expression::operators::Eq<
                  community_follower::columns::user_id,
                  diesel::expression::bound::Bound<Integer, i32>,
                >,
              >,
            >,
            comment_saved::table,
            LeftOuter,
          >,
          diesel::expression::operators::And<
            diesel::expression::operators::Eq<
              comment::columns::id,
              comment_saved::columns::comment_id,
            >,
            diesel::expression::operators::Eq<
              comment_saved::columns::user_id,
              diesel::expression::bound::Bound<Integer, i32>,
            >,
          >,
        >,
        comment_like::table,
        LeftOuter,
      >,
      diesel::expression::operators::And<
        diesel::expression::operators::Eq<comment::columns::id, comment_like::columns::comment_id>,
        diesel::expression::operators::Eq<
          comment_like::columns::user_id,
          diesel::expression::bound::Bound<Integer, i32>,
        >,
      >,
    >,
    Pg,
  >;
}

pub struct CommentQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: join_types::BoxedCommentJoin<'a>,
  listing_type: ListingType,
  sort: &'a SortType,
  for_community_id: Option<i32>,
  for_community_name: Option<String>,
  for_post_id: Option<i32>,
  for_creator_id: Option<i32>,
  for_recipient_id: Option<i32>,
  search_term: Option<String>,
  saved_only: bool,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommentQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_user_id: Option<i32>) -> Self {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let query = comment::table
      .inner_join(user_::table)
      // recipient here
      .left_join(comment_alias_1::table.on(comment_alias_1::id.nullable().eq(comment::parent_id)))
      .left_join(user_alias_1::table.on(user_alias_1::id.eq(comment_alias_1::creator_id)))
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
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
        comment::all_columns,
        User_::safe_columns_tuple(),
        comment_alias_1::all_columns.nullable(),
        UserAlias1::safe_columns_tuple().nullable(),
        post::all_columns,
        Community::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    CommentQueryBuilder {
      conn,
      query,
      listing_type: ListingType::All,
      sort: &SortType::New,
      for_community_id: None,
      for_community_name: None,
      for_post_id: None,
      for_creator_id: None,
      for_recipient_id: None,
      search_term: None,
      saved_only: false,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn listing_type(mut self, listing_type: ListingType) -> Self {
    self.listing_type = listing_type;
    self
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn for_post_id<T: MaybeOptional<i32>>(mut self, for_post_id: T) -> Self {
    self.for_post_id = for_post_id.get_optional();
    self
  }

  pub fn for_creator_id<T: MaybeOptional<i32>>(mut self, for_creator_id: T) -> Self {
    self.for_creator_id = for_creator_id.get_optional();
    self
  }

  pub fn for_recipient_id<T: MaybeOptional<i32>>(mut self, for_recipient_id: T) -> Self {
    self.for_creator_id = for_recipient_id.get_optional();
    self
  }

  pub fn for_community_id<T: MaybeOptional<i32>>(mut self, for_community_id: T) -> Self {
    self.for_community_id = for_community_id.get_optional();
    self
  }

  pub fn for_community_name<T: MaybeOptional<String>>(mut self, for_community_name: T) -> Self {
    self.for_community_name = for_community_name.get_optional();
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
    self
  }

  pub fn saved_only(mut self, saved_only: bool) -> Self {
    self.saved_only = saved_only;
    self
  }

  pub fn unread_only(mut self, unread_only: bool) -> Self {
    self.unread_only = unread_only;
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

  pub fn list(self) -> Result<Vec<CommentView>, Error> {
    use diesel::dsl::*;

    let mut query = self.query;

    // The replies
    if let Some(for_recipient_id) = self.for_recipient_id {
      query = query
        // TODO needs lots of testing
        .filter(user_alias_1::id.eq(for_recipient_id))
        .filter(comment::deleted.eq(false))
        .filter(comment::removed.eq(false));
    }

    if self.unread_only {
      query = query.filter(comment::read.eq(false));
    }

    if let Some(for_creator_id) = self.for_creator_id {
      query = query.filter(comment::creator_id.eq(for_creator_id));
    };

    if let Some(for_community_id) = self.for_community_id {
      query = query.filter(post::community_id.eq(for_community_id));
    }

    if let Some(for_community_name) = self.for_community_name {
      query = query
        .filter(community::name.eq(for_community_name))
        .filter(comment::local.eq(true));
    }

    if let Some(for_post_id) = self.for_post_id {
      query = query.filter(comment::post_id.eq(for_post_id));
    };

    if let Some(search_term) = self.search_term {
      query = query.filter(comment::content.ilike(fuzzy_search(&search_term)));
    };

    query = match self.listing_type {
      // ListingType::Subscribed => query.filter(community_follower::subscribed.eq(true)),
      ListingType::Subscribed => query.filter(community_follower::user_id.is_not_null()), // TODO could be this: and(community_follower::user_id.eq(user_id_join)),
      ListingType::Local => query.filter(community::local.eq(true)),
      _ => query,
    };

    if self.saved_only {
      query = query.filter(comment_saved::id.is_not_null());
    }

    query = match self.sort {
      SortType::Hot | SortType::Active => query
        .order_by(hot_rank(comment_aggregates::score, comment::published).desc())
        .then_order_by(comment::published.desc()),
      SortType::New => query.order_by(comment::published.desc()),
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

    // Note: deleted and removed comments are done on the front side
    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentViewTuple>(self.conn)?;

    Ok(CommentView::to_vec(res))
  }
}

impl ViewToVec for CommentView {
  type DbTuple = CommentViewTuple;
  fn to_vec(posts: Vec<Self::DbTuple>) -> Vec<Self> {
    posts
      .iter()
      .map(|a| Self {
        comment: a.0.to_owned(),
        creator: a.1.to_owned(),
        recipient: a.3.to_owned(),
        post: a.4.to_owned(),
        community: a.5.to_owned(),
        counts: a.6.to_owned(),
        creator_banned_from_community: a.7.is_some(),
        subscribed: a.8.is_some(),
        saved: a.9.is_some(),
        my_vote: a.10,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{comment::*, community::*, post::*, user::*},
    tests::establish_unpooled_connection,
    views::comment_view::*,
    Crud,
    Likeable,
    *,
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "timmy".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: "test community 5".to_string(),
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
      creator_id: inserted_user.id,
      removed: None,
      deleted: None,
      updated: None,
      nsfw: false,
      actor_id: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
      icon: None,
      banner: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post 2".into(),
      creator_id: inserted_user.id,
      url: None,
      body: None,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      stickied: None,
      updated: None,
      nsfw: false,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment 32".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      parent_id: None,
      removed: None,
      deleted: None,
      read: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let inserted_comment = Comment::create(&conn, &comment_form).unwrap();

    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(&conn, &comment_like_form).unwrap();

    let expected_comment_view_no_user = CommentView {
      creator_banned_from_community: false,
      my_vote: None,
      subscribed: false,
      saved: false,
      comment: Comment {
        id: inserted_comment.id,
        content: "A test comment 32".into(),
        creator_id: inserted_user.id,
        post_id: inserted_post.id,
        parent_id: None,
        removed: false,
        deleted: false,
        read: false,
        published: inserted_comment.published,
        ap_id: inserted_comment.ap_id,
        updated: None,
        local: true,
      },
      creator: UserSafe {
        id: inserted_user.id,
        name: "timmy".into(),
        preferred_username: None,
        published: inserted_user.published,
        avatar: None,
        actor_id: inserted_user.actor_id.to_owned(),
        local: true,
        banned: false,
        deleted: false,
        bio: None,
        banner: None,
        admin: false,
        updated: None,
        matrix_user_id: None,
      },
      recipient: None,
      post: Post {
        id: inserted_post.id,
        name: inserted_post.name.to_owned(),
        creator_id: inserted_user.id,
        url: None,
        body: None,
        published: inserted_post.published,
        updated: None,
        community_id: inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        stickied: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_html: None,
        thumbnail_url: None,
        ap_id: inserted_post.ap_id.to_owned(),
        local: true,
      },
      community: CommunitySafe {
        id: inserted_community.id,
        name: "test community 5".to_string(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.to_owned(),
        local: true,
        title: "nada".to_owned(),
        description: None,
        creator_id: inserted_user.id,
        category_id: 1,
        updated: None,
        banner: None,
        published: inserted_community.published,
      },
      counts: CommentAggregates {
        id: inserted_comment.id, // TODO
        comment_id: inserted_comment.id,
        score: 1,
        upvotes: 1,
        downvotes: 0,
      },
    };

    let mut expected_comment_view_with_user = expected_comment_view_no_user.to_owned();
    expected_comment_view_with_user.my_vote = Some(1);

    let read_comment_views_no_user = CommentQueryBuilder::create(&conn, None)
      .for_post_id(inserted_post.id)
      .list()
      .unwrap();

    let read_comment_views_with_user = CommentQueryBuilder::create(&conn, Some(inserted_user.id))
      .for_post_id(inserted_post.id)
      .list()
      .unwrap();

    let like_removed = CommentLike::remove(&conn, inserted_user.id, inserted_comment.id).unwrap();
    let num_deleted = Comment::delete(&conn, inserted_comment.id).unwrap();
    Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_comment_view_no_user, read_comment_views_no_user[0]);
    assert_eq!(
      expected_comment_view_with_user,
      read_comment_views_with_user[0]
    );
    assert_eq!(1, num_deleted);
    assert_eq!(1, like_removed);
  }
}
