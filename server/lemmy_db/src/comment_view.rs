// TODO, remove the cross join here, just join to user directly
use crate::{fuzzy_search, limit_and_offset, ListingType, MaybeOptional, SortType};
use diesel::{dsl::*, pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

// The faked schema since diesel doesn't do views
table! {
  comment_view (id) {
    id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    post_name -> Varchar,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    ap_id -> Text,
    local -> Bool,
    community_id -> Int4,
    community_actor_id -> Text,
    community_local -> Bool,
    community_name -> Varchar,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_actor_id -> Text,
    creator_local -> Bool,
    creator_name -> Varchar,
    creator_published -> Timestamp,
    creator_avatar -> Nullable<Text>,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    hot_rank -> Int4,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    subscribed -> Nullable<Bool>,
    saved -> Nullable<Bool>,
  }
}

table! {
  comment_fast_view (id) {
    id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    post_name -> Varchar,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    ap_id -> Text,
    local -> Bool,
    community_id -> Int4,
    community_actor_id -> Text,
    community_local -> Bool,
    community_name -> Varchar,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_actor_id -> Text,
    creator_local -> Bool,
    creator_name -> Varchar,
    creator_published -> Timestamp,
    creator_avatar -> Nullable<Text>,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    hot_rank -> Int4,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    subscribed -> Nullable<Bool>,
    saved -> Nullable<Bool>,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "comment_fast_view"]
pub struct CommentView {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub post_name: String,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: String,
  pub local: bool,
  pub community_id: i32,
  pub community_actor_id: String,
  pub community_local: bool,
  pub community_name: String,
  pub banned: bool,
  pub banned_from_community: bool,
  pub creator_actor_id: String,
  pub creator_local: bool,
  pub creator_name: String,
  pub creator_published: chrono::NaiveDateTime,
  pub creator_avatar: Option<String>,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub hot_rank: i32,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub subscribed: Option<bool>,
  pub saved: Option<bool>,
}

pub struct CommentQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: super::comment_view::comment_fast_view::BoxedQuery<'a, Pg>,
  listing_type: ListingType,
  sort: &'a SortType,
  for_community_id: Option<i32>,
  for_post_id: Option<i32>,
  for_creator_id: Option<i32>,
  search_term: Option<String>,
  my_user_id: Option<i32>,
  saved_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommentQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::comment_view::comment_fast_view::dsl::*;

    let query = comment_fast_view.into_boxed();

    CommentQueryBuilder {
      conn,
      query,
      listing_type: ListingType::All,
      sort: &SortType::New,
      for_community_id: None,
      for_post_id: None,
      for_creator_id: None,
      search_term: None,
      my_user_id: None,
      saved_only: false,
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

  pub fn for_community_id<T: MaybeOptional<i32>>(mut self, for_community_id: T) -> Self {
    self.for_community_id = for_community_id.get_optional();
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
    self
  }

  pub fn my_user_id<T: MaybeOptional<i32>>(mut self, my_user_id: T) -> Self {
    self.my_user_id = my_user_id.get_optional();
    self
  }

  pub fn saved_only(mut self, saved_only: bool) -> Self {
    self.saved_only = saved_only;
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
    use super::comment_view::comment_fast_view::dsl::*;

    let mut query = self.query;

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(my_user_id) = self.my_user_id {
      query = query.filter(user_id.eq(my_user_id));
    } else {
      query = query.filter(user_id.is_null());
    }

    if let Some(for_creator_id) = self.for_creator_id {
      query = query.filter(creator_id.eq(for_creator_id));
    };

    if let Some(for_community_id) = self.for_community_id {
      query = query.filter(community_id.eq(for_community_id));
    }

    if let Some(for_post_id) = self.for_post_id {
      query = query.filter(post_id.eq(for_post_id));
    };

    if let Some(search_term) = self.search_term {
      query = query.filter(content.ilike(fuzzy_search(&search_term)));
    };

    if let ListingType::Subscribed = self.listing_type {
      query = query.filter(subscribed.eq(true));
    }

    if self.saved_only {
      query = query.filter(saved.eq(true));
    }

    query = match self.sort {
      SortType::Hot => query
        .order_by(hot_rank.desc())
        .then_order_by(published.desc()),
      SortType::New => query.order_by(published.desc()),
      SortType::TopAll => query.order_by(score.desc()),
      SortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(score.desc()),
      SortType::TopMonth => query
        .filter(published.gt(now - 1.months()))
        .order_by(score.desc()),
      SortType::TopWeek => query
        .filter(published.gt(now - 1.weeks()))
        .order_by(score.desc()),
      SortType::TopDay => query
        .filter(published.gt(now - 1.days()))
        .order_by(score.desc()),
      // _ => query.order_by(published.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    // Note: deleted and removed comments are done on the front side
    query
      .limit(limit)
      .offset(offset)
      .load::<CommentView>(self.conn)
  }
}

impl CommentView {
  pub fn read(
    conn: &PgConnection,
    from_comment_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    use super::comment_view::comment_fast_view::dsl::*;
    let mut query = comment_fast_view.into_boxed();

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(my_user_id) = my_user_id {
      query = query.filter(user_id.eq(my_user_id));
    } else {
      query = query.filter(user_id.is_null());
    }

    query = query
      .filter(id.eq(from_comment_id))
      .order_by(published.desc());

    query.first::<Self>(conn)
  }
}

// The faked schema since diesel doesn't do views
table! {
  reply_fast_view (id) {
    id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    post_name -> Varchar,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    ap_id -> Text,
    local -> Bool,
    community_id -> Int4,
    community_actor_id -> Text,
    community_local -> Bool,
    community_name -> Varchar,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_actor_id -> Text,
    creator_local -> Bool,
    creator_name -> Varchar,
    creator_avatar -> Nullable<Text>,
    creator_published -> Timestamp,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    hot_rank -> Int4,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    subscribed -> Nullable<Bool>,
    saved -> Nullable<Bool>,
    recipient_id -> Int4,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "reply_fast_view"]
pub struct ReplyView {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub post_name: String,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: String,
  pub local: bool,
  pub community_id: i32,
  pub community_actor_id: String,
  pub community_local: bool,
  pub community_name: String,
  pub banned: bool,
  pub banned_from_community: bool,
  pub creator_actor_id: String,
  pub creator_local: bool,
  pub creator_name: String,
  pub creator_avatar: Option<String>,
  pub creator_published: chrono::NaiveDateTime,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub hot_rank: i32,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub subscribed: Option<bool>,
  pub saved: Option<bool>,
  pub recipient_id: i32,
}

pub struct ReplyQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: super::comment_view::reply_fast_view::BoxedQuery<'a, Pg>,
  for_user_id: i32,
  sort: &'a SortType,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> ReplyQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, for_user_id: i32) -> Self {
    use super::comment_view::reply_fast_view::dsl::*;

    let query = reply_fast_view.into_boxed();

    ReplyQueryBuilder {
      conn,
      query,
      for_user_id,
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

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<ReplyView>, Error> {
    use super::comment_view::reply_fast_view::dsl::*;

    let mut query = self.query;

    query = query
      .filter(user_id.eq(self.for_user_id))
      .filter(recipient_id.eq(self.for_user_id))
      .filter(deleted.eq(false))
      .filter(removed.eq(false));

    if self.unread_only {
      query = query.filter(read.eq(false));
    }

    query = match self.sort {
      // SortType::Hot => query.order_by(hot_rank.desc()),
      SortType::New => query.order_by(published.desc()),
      SortType::TopAll => query.order_by(score.desc()),
      SortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(score.desc()),
      SortType::TopMonth => query
        .filter(published.gt(now - 1.months()))
        .order_by(score.desc()),
      SortType::TopWeek => query
        .filter(published.gt(now - 1.weeks()))
        .order_by(score.desc()),
      SortType::TopDay => query
        .filter(published.gt(now - 1.days()))
        .order_by(score.desc()),
      _ => query.order_by(published.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    query
      .limit(limit)
      .offset(offset)
      .load::<ReplyView>(self.conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    comment::*,
    comment_view::*,
    community::*,
    post::*,
    tests::establish_unpooled_connection,
    user::*,
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
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: "http://fake.com".into(),
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
      actor_id: "http://fake.com".into(),
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
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
      ap_id: "http://fake.com".into(),
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
      ap_id: "http://fake.com".into(),
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
      id: inserted_comment.id,
      content: "A test comment 32".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      post_name: inserted_post.name.to_owned(),
      community_id: inserted_community.id,
      community_name: inserted_community.name.to_owned(),
      parent_id: None,
      removed: false,
      deleted: false,
      read: false,
      banned: false,
      banned_from_community: false,
      published: inserted_comment.published,
      updated: None,
      creator_name: inserted_user.name.to_owned(),
      creator_published: inserted_user.published,
      creator_avatar: None,
      score: 1,
      downvotes: 0,
      hot_rank: 0,
      upvotes: 1,
      user_id: None,
      my_vote: None,
      subscribed: None,
      saved: None,
      ap_id: "http://fake.com".to_string(),
      local: true,
      community_actor_id: inserted_community.actor_id.to_owned(),
      community_local: true,
      creator_actor_id: inserted_user.actor_id.to_owned(),
      creator_local: true,
    };

    let expected_comment_view_with_user = CommentView {
      id: inserted_comment.id,
      content: "A test comment 32".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      post_name: inserted_post.name.to_owned(),
      community_id: inserted_community.id,
      community_name: inserted_community.name.to_owned(),
      parent_id: None,
      removed: false,
      deleted: false,
      read: false,
      banned: false,
      banned_from_community: false,
      published: inserted_comment.published,
      updated: None,
      creator_name: inserted_user.name.to_owned(),
      creator_published: inserted_user.published,
      creator_avatar: None,
      score: 1,
      downvotes: 0,
      hot_rank: 0,
      upvotes: 1,
      user_id: Some(inserted_user.id),
      my_vote: Some(1),
      subscribed: Some(false),
      saved: Some(false),
      ap_id: "http://fake.com".to_string(),
      local: true,
      community_actor_id: inserted_community.actor_id.to_owned(),
      community_local: true,
      creator_actor_id: inserted_user.actor_id.to_owned(),
      creator_local: true,
    };

    let mut read_comment_views_no_user = CommentQueryBuilder::create(&conn)
      .for_post_id(inserted_post.id)
      .list()
      .unwrap();
    read_comment_views_no_user[0].hot_rank = 0;

    let mut read_comment_views_with_user = CommentQueryBuilder::create(&conn)
      .for_post_id(inserted_post.id)
      .my_user_id(inserted_user.id)
      .list()
      .unwrap();
    read_comment_views_with_user[0].hot_rank = 0;

    let like_removed = CommentLike::remove(&conn, &comment_like_form).unwrap();
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
