extern crate diesel;
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

#[derive(EnumString,ToString,Debug, Serialize, Deserialize)]
pub enum ListingType {
  All, Subscribed, Community
}

#[derive(EnumString,ToString,Debug, Serialize, Deserialize)]
pub enum ListingSortType {
  Hot, New, TopDay, TopWeek, TopMonth, TopYear, TopAll
}

// The faked schema since diesel doesn't do views
table! {
  post_view (id) {
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    id -> Int4,
    name -> Varchar,
    url -> Nullable<Text>,
    body -> Nullable<Text>,
    creator_id -> Int4,
    creator_name -> Varchar,
    community_id -> Int4,
    community_name -> Varchar,
    number_of_comments -> BigInt,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    hot_rank -> Int4,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}


#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName)]
#[table_name="post_view"]
pub struct PostView {
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub creator_name: String,
  pub community_id: i32,
  pub community_name: String,
  pub number_of_comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub hot_rank: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

impl PostView {
  pub fn list(conn: &PgConnection, type_: ListingType, sort: ListingSortType, from_community_id: Option<i32>, from_user_id: Option<i32>, limit: i64) -> Result<Vec<Self>, Error> {
    use actions::post_view::post_view::dsl::*;
    use diesel::dsl::*;
    use diesel::prelude::*;

    let mut query = post_view.limit(limit).into_boxed();

    if let Some(from_community_id) = from_community_id {
      query = query.filter(community_id.eq(from_community_id));
    };

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(from_user_id) = from_user_id {
      query = query.filter(user_id.eq(from_user_id));
    } else {
      query = query.filter(user_id.is_null());
    }

    query = match sort {
      ListingSortType::Hot => query.order_by(hot_rank.desc()),
      ListingSortType::New => query.order_by(published.desc()),
      ListingSortType::TopAll => query.order_by(score.desc()),
      ListingSortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(score.desc()),
        ListingSortType::TopMonth => query
          .filter(published.gt(now - 1.months()))
          .order_by(score.desc()),
          ListingSortType::TopWeek => query
            .filter(published.gt(now - 1.weeks()))
            .order_by(score.desc()),
            ListingSortType::TopDay => query
              .filter(published.gt(now - 1.days()))
              .order_by(score.desc())
    };

    query.load::<Self>(conn) 
  }


  pub fn get(conn: &PgConnection, from_post_id: i32, from_user_id: Option<i32>) -> Result<Self, Error> {

    use actions::post_view::post_view::dsl::*;
    use diesel::dsl::*;
    use diesel::prelude::*;

    let mut query = post_view.into_boxed();

    query = query.filter(id.eq(from_post_id));

    if let Some(from_user_id) = from_user_id {
      query = query.filter(user_id.eq(from_user_id));
    } else {
      // This fills in nulls for the user_id and user vote
      query = query
        .select((
            sql("null"),
            sql("null"),
            id, 
            name, 
            url, 
            body, 
            creator_id, 
            creator_name, 
            community_id, 
            community_name, 
            number_of_comments, 
            score, 
            upvotes, 
            downvotes, 
            hot_rank, 
            published, 
            updated
            ))
        .group_by((
            id,
            name,
            url,
            body, 
            creator_id,
            creator_name,
            community_id,
            community_name,
            number_of_comments,
            score,
            upvotes,
            downvotes,
            hot_rank,
            published,
            updated
            ));
    };

    query.first::<Self>(conn)
  }
}



#[cfg(test)]
mod tests {
  use {establish_connection, Crud, Likeable};
  use super::*;
  use actions::community::*;
  use actions::user::*;
  use actions::post::*;
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
      updated: None
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: community_name.to_owned(),
      creator_id: inserted_user.id,
      updated: None
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: post_name.to_owned(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
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
      community_name: community_name.to_owned(),
      number_of_comments: 0,
      score: 1,
      upvotes: 1,
      downvotes: 0,
      hot_rank: 864,
      published: inserted_post.published,
      updated: None
    };

    let expected_post_listing_with_user = PostView {
      user_id: Some(inserted_user.id),
      my_vote: Some(1),
      id: inserted_post.id,
      name: post_name.to_owned(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      creator_name: user_name.to_owned(),
      community_id: inserted_community.id,
      community_name: community_name.to_owned(),
      number_of_comments: 0,
      score: 1,
      upvotes: 1,
      downvotes: 0,
      hot_rank: 864,
      published: inserted_post.published,
      updated: None
    };


    let read_post_listings_with_user = PostView::list(&conn, ListingType::Community, ListingSortType::New, Some(inserted_community.id), Some(inserted_user.id), 10).unwrap();
    let read_post_listings_no_user = PostView::list(&conn, ListingType::Community, ListingSortType::New, Some(inserted_community.id), None, 10).unwrap();
    let read_post_listing_no_user = PostView::get(&conn, inserted_post.id, None).unwrap();
    let read_post_listing_with_user = PostView::get(&conn, inserted_post.id, Some(inserted_user.id)).unwrap();

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
