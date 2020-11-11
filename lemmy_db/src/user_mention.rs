use super::comment::Comment;
use crate::{schema::user_mention, Crud};
use diesel::{dsl::*, result::Error, *};

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug)]
#[belongs_to(Comment)]
#[table_name = "user_mention"]
pub struct UserMention {
  pub id: i32,
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "user_mention"]
pub struct UserMentionForm {
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: Option<bool>,
}

impl Crud<UserMentionForm> for UserMention {
  fn read(conn: &PgConnection, user_mention_id: i32) -> Result<Self, Error> {
    use crate::schema::user_mention::dsl::*;
    user_mention.find(user_mention_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, user_mention_form: &UserMentionForm) -> Result<Self, Error> {
    use crate::schema::user_mention::dsl::*;
    insert_into(user_mention)
      .values(user_mention_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    user_mention_id: i32,
    user_mention_form: &UserMentionForm,
  ) -> Result<Self, Error> {
    use crate::schema::user_mention::dsl::*;
    diesel::update(user_mention.find(user_mention_id))
      .set(user_mention_form)
      .get_result::<Self>(conn)
  }
}

impl UserMention {
  pub fn update_read(
    conn: &PgConnection,
    user_mention_id: i32,
    new_read: bool,
  ) -> Result<Self, Error> {
    use crate::schema::user_mention::dsl::*;
    diesel::update(user_mention.find(user_mention_id))
      .set(read.eq(new_read))
      .get_result::<Self>(conn)
  }

  pub fn mark_all_as_read(conn: &PgConnection, for_recipient_id: i32) -> Result<Vec<Self>, Error> {
    use crate::schema::user_mention::dsl::*;
    diesel::update(
      user_mention
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    comment::*,
    community::*,
    post::*,
    tests::establish_unpooled_connection,
    user::*,
    user_mention::*,
    ListingType,
    SortType,
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "terrylake".into(),
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

    let recipient_form = UserForm {
      name: "terrylakes recipient".into(),
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

    let inserted_recipient = User_::create(&conn, &recipient_form).unwrap();

    let new_community = CommunityForm {
      name: "test community lake".to_string(),
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
      name: "A test post".into(),
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
      content: "A test comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      removed: None,
      deleted: None,
      read: None,
      parent_id: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let inserted_comment = Comment::create(&conn, &comment_form).unwrap();

    let user_mention_form = UserMentionForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_mention = UserMention::create(&conn, &user_mention_form).unwrap();

    let expected_mention = UserMention {
      id: inserted_mention.id,
      recipient_id: inserted_mention.recipient_id,
      comment_id: inserted_mention.comment_id,
      read: false,
      published: inserted_mention.published,
    };

    let read_mention = UserMention::read(&conn, inserted_mention.id).unwrap();
    let updated_mention =
      UserMention::update(&conn, inserted_mention.id, &user_mention_form).unwrap();
    Comment::delete(&conn, inserted_comment.id).unwrap();
    Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();
    User_::delete(&conn, inserted_recipient.id).unwrap();

    assert_eq!(expected_mention, read_mention);
    assert_eq!(expected_mention, inserted_mention);
    assert_eq!(expected_mention, updated_mention);
  }
}
