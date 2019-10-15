use super::*;

// The faked schema since diesel doesn't do views
table! {
  comment_view (id) {
    id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    community_id -> Int4,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_name -> Varchar,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    saved -> Nullable<Bool>,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "comment_view"]
pub struct CommentView {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub community_id: i32,
  pub banned: bool,
  pub banned_from_community: bool,
  pub creator_name: String,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub saved: Option<bool>,
}

impl CommentView {
  pub fn list(
    conn: &PgConnection,
    sort: &SortType,
    for_post_id: Option<i32>,
    for_creator_id: Option<i32>,
    search_term: Option<String>,
    my_user_id: Option<i32>,
    saved_only: bool,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    use super::comment_view::comment_view::dsl::*;

    let (limit, offset) = limit_and_offset(page, limit);

    // TODO no limits here?
    let mut query = comment_view.into_boxed();

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(my_user_id) = my_user_id {
      query = query.filter(user_id.eq(my_user_id));
    } else {
      query = query.filter(user_id.is_null());
    }

    if let Some(for_creator_id) = for_creator_id {
      query = query.filter(creator_id.eq(for_creator_id));
    };

    if let Some(for_post_id) = for_post_id {
      query = query.filter(post_id.eq(for_post_id));
    };

    if let Some(search_term) = search_term {
      query = query.filter(content.ilike(fuzzy_search(&search_term)));
    };

    if saved_only {
      query = query.filter(saved.eq(true));
    }

    query = match sort {
      // SortType::Hot => query.order(hot_rank.desc(), published.desc()),
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

    // Note: deleted and removed comments are done on the front side
    query.limit(limit).offset(offset).load::<Self>(conn)
  }

  pub fn read(
    conn: &PgConnection,
    from_comment_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    use super::comment_view::comment_view::dsl::*;

    let mut query = comment_view.into_boxed();

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
  reply_view (id) {
    id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    community_id -> Int4,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_name -> Varchar,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    saved -> Nullable<Bool>,
    recipient_id -> Int4,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "reply_view"]
pub struct ReplyView {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub community_id: i32,
  pub banned: bool,
  pub banned_from_community: bool,
  pub creator_name: String,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub saved: Option<bool>,
  pub recipient_id: i32,
}

impl ReplyView {
  pub fn get_replies(
    conn: &PgConnection,
    for_user_id: i32,
    sort: &SortType,
    unread_only: bool,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    use super::comment_view::reply_view::dsl::*;

    let (limit, offset) = limit_and_offset(page, limit);

    let mut query = reply_view.into_boxed();

    query = query
      .filter(user_id.eq(for_user_id))
      .filter(recipient_id.eq(for_user_id));

    if unread_only {
      query = query.filter(read.eq(false));
    }

    query = match sort {
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

    query.limit(limit).offset(offset).load::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use super::super::comment::*;
  use super::super::community::*;
  use super::super::post::*;
  use super::super::user::*;
  use super::*;
  #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_user = UserForm {
      name: "timmy".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
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
      updated: None,
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
      community_id: inserted_community.id,
      parent_id: None,
      removed: false,
      deleted: false,
      read: false,
      banned: false,
      banned_from_community: false,
      published: inserted_comment.published,
      updated: None,
      creator_name: inserted_user.name.to_owned(),
      score: 1,
      downvotes: 0,
      upvotes: 1,
      user_id: None,
      my_vote: None,
      saved: None,
    };

    let expected_comment_view_with_user = CommentView {
      id: inserted_comment.id,
      content: "A test comment 32".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      community_id: inserted_community.id,
      parent_id: None,
      removed: false,
      deleted: false,
      read: false,
      banned: false,
      banned_from_community: false,
      published: inserted_comment.published,
      updated: None,
      creator_name: inserted_user.name.to_owned(),
      score: 1,
      downvotes: 0,
      upvotes: 1,
      user_id: Some(inserted_user.id),
      my_vote: Some(1),
      saved: None,
    };

    let read_comment_views_no_user = CommentView::list(
      &conn,
      &SortType::New,
      Some(inserted_post.id),
      None,
      None,
      None,
      false,
      None,
      None,
    )
    .unwrap();
    let read_comment_views_with_user = CommentView::list(
      &conn,
      &SortType::New,
      Some(inserted_post.id),
      None,
      None,
      Some(inserted_user.id),
      false,
      None,
      None,
    )
    .unwrap();
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
