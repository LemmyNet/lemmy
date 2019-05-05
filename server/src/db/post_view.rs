use super::*;

#[derive(EnumString,ToString,Debug, Serialize, Deserialize)]
pub enum PostListingType {
  All, Subscribed, Community
}

// The faked schema since diesel doesn't do views
table! {
  post_view (id) {
    id -> Int4,
    name -> Varchar,
    url -> Nullable<Text>,
    body -> Nullable<Text>,
    creator_id -> Int4,
    community_id -> Int4,
    removed -> Bool,
    locked -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    creator_name -> Varchar,
    community_name -> Varchar,
    community_removed -> Bool,
    community_deleted -> Bool,
    number_of_comments -> BigInt,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    hot_rank -> Int4,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    subscribed -> Nullable<Bool>,
    read -> Nullable<Bool>,
    saved -> Nullable<Bool>,
  }
}


#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="post_view"]
pub struct PostView {
  pub id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub community_id: i32,
  pub removed: bool,
  pub locked: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub creator_name: String,
  pub community_name: String,
  pub community_removed: bool,
  pub community_deleted: bool,
  pub number_of_comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub hot_rank: i32,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub subscribed: Option<bool>,
  pub read: Option<bool>,
  pub saved: Option<bool>,
}

impl PostView {
  pub fn list(conn: &PgConnection, 
              type_: PostListingType, 
              sort: &SortType, 
              for_community_id: Option<i32>, 
              for_creator_id: Option<i32>, 
              search_term: Option<String>,
              my_user_id: Option<i32>, 
              saved_only: bool,
              unread_only: bool,
              page: Option<i64>,
              limit: Option<i64>,
              ) -> Result<Vec<Self>, Error> {
    use super::post_view::post_view::dsl::*;

    let (limit, offset) = limit_and_offset(page, limit);

    let mut query = post_view.into_boxed();

    if let Some(for_community_id) = for_community_id {
      query = query.filter(community_id.eq(for_community_id));
    };

    if let Some(for_creator_id) = for_creator_id {
      query = query.filter(creator_id.eq(for_creator_id));
    };

    if let Some(search_term) = search_term {
      query = query.filter(name.ilike(fuzzy_search(&search_term)));
    };

    // TODO these are wrong, bc they'll only show saved for your logged in user, not theirs
    if saved_only {
      query = query.filter(saved.eq(true));
    };

    if unread_only {
      query = query.filter(read.eq(false));
    };

    match type_ {
      PostListingType::Subscribed  => {
        query = query.filter(subscribed.eq(true));
      },
      _ => {}
    };

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(my_user_id) = my_user_id {
      query = query.filter(user_id.eq(my_user_id));
    } else {
      query = query.filter(user_id.is_null());
    }

    query = match sort {
      SortType::Hot => query.order_by(hot_rank.desc()),
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
              .order_by(score.desc())
    };

    query = query
      .limit(limit)
      .offset(offset)
      .filter(removed.eq(false))
      .filter(deleted.eq(false))
      .filter(community_removed.eq(false))
      .filter(community_deleted.eq(false));

    query.load::<Self>(conn) 
  }


  pub fn read(conn: &PgConnection, from_post_id: i32, my_user_id: Option<i32>) -> Result<Self, Error> {

    use super::post_view::post_view::dsl::*;
    use diesel::prelude::*;

    let mut query = post_view.into_boxed();

    query = query.filter(id.eq(from_post_id));

    if let Some(my_user_id) = my_user_id {
      query = query.filter(user_id.eq(my_user_id));
    } else {
      query = query.filter(user_id.is_null());
    };

    query.first::<Self>(conn)
  }
}



#[cfg(test)]
mod tests {
  use super::*;
  use super::super::community::*;
  use super::super::user::*;
  use super::super::post::*;
  #[test]
  fn test_crud() {
    let conn = establish_connection();

    let user_name = "tegan".to_string();
    let community_name = "test_community_3".to_string();
    let post_name = "test post 3".to_string();

    let new_user = UserForm {
      name: user_name.to_owned(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      updated: None,
      admin: false,
      banned: false,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: community_name.to_owned(),
      title: "nada".to_owned(),
      description: None,
      creator_id: inserted_user.id,
      category_id: 1,
      removed: None,
      deleted: None,
      updated: None
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: post_name.to_owned(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      updated: None
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1
    };

    let inserted_post_like = PostLike::like(&conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_like.published,
      score: 1
    };

    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1
    };

    // the non user version
    let expected_post_listing_no_user = PostView {
      user_id: None,
      my_vote: None,
      id: inserted_post.id,
      name: post_name.to_owned(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      creator_name: user_name.to_owned(),
      community_id: inserted_community.id,
      removed: false,
      deleted: false,
      locked: false,
      community_name: community_name.to_owned(),
      community_removed: false,
      community_deleted: false,
      number_of_comments: 0,
      score: 1,
      upvotes: 1,
      downvotes: 0,
      hot_rank: 1728,
      published: inserted_post.published,
      updated: None,
      subscribed: None,
      read: None,
      saved: None,
    };

    let expected_post_listing_with_user = PostView {
      user_id: Some(inserted_user.id),
      my_vote: Some(1),
      id: inserted_post.id,
      name: post_name.to_owned(),
      url: None,
      body: None,
      removed: false,
      deleted: false,
      locked: false,
      creator_id: inserted_user.id,
      creator_name: user_name.to_owned(),
      community_id: inserted_community.id,
      community_name: community_name.to_owned(),
      community_removed: false,
      community_deleted: false,
      number_of_comments: 0,
      score: 1,
      upvotes: 1,
      downvotes: 0,
      hot_rank: 1728,
      published: inserted_post.published,
      updated: None,
      subscribed: None,
      read: None,
      saved: None,
    };


    let read_post_listings_with_user = PostView::list(&conn, 
                                                      PostListingType::Community, 
                                                      &SortType::New, Some(inserted_community.id), 
                                                      None, 
                                                      None,
                                                      Some(inserted_user.id), 
                                                      false, 
                                                      false, 
                                                      None, 
                                                      None).unwrap();
    let read_post_listings_no_user = PostView::list(&conn, 
                                                    PostListingType::Community, 
                                                    &SortType::New, 
                                                    Some(inserted_community.id), 
                                                    None, 
                                                    None, 
                                                    None,
                                                    false, 
                                                    false, 
                                                    None, 
                                                    None).unwrap();
    let read_post_listing_no_user = PostView::read(&conn, inserted_post.id, None).unwrap();
    let read_post_listing_with_user = PostView::read(&conn, inserted_post.id, Some(inserted_user.id)).unwrap();

    let like_removed = PostLike::remove(&conn, &post_like_form).unwrap();
    let num_deleted = Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    // The with user
    assert_eq!(expected_post_listing_with_user, read_post_listings_with_user[0]);
    assert_eq!(expected_post_listing_with_user, read_post_listing_with_user);
    assert_eq!(1, read_post_listings_with_user.len());

    // Without the user
    assert_eq!(expected_post_listing_no_user, read_post_listings_no_user[0]);
    assert_eq!(expected_post_listing_no_user, read_post_listing_no_user);
    assert_eq!(1, read_post_listings_no_user.len());

    // assert_eq!(expected_post, inserted_post);
    // assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);
  }
}
