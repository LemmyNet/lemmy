use crate::{
  newtypes::{CommentId, CommentReplyId, PersonId},
  source::comment_reply::*,
  traits::Crud,
};
use diesel::{dsl::*, result::Error, *};

impl Crud for CommentReply {
  type InsertForm = CommentReplyInsertForm;
  type UpdateForm = CommentReplyUpdateForm;
  type IdType = CommentReplyId;
  fn read(conn: &mut PgConnection, comment_reply_id: CommentReplyId) -> Result<Self, Error> {
    use crate::schema::comment_reply::dsl::*;
    comment_reply.find(comment_reply_id).first::<Self>(conn)
  }

  fn create(conn: &mut PgConnection, comment_reply_form: &Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::comment_reply::dsl::*;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesnt return the existing row here
    insert_into(comment_reply)
      .values(comment_reply_form)
      .on_conflict((recipient_id, comment_id))
      .do_update()
      .set(comment_reply_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &mut PgConnection,
    comment_reply_id: CommentReplyId,
    comment_reply_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    use crate::schema::comment_reply::dsl::*;
    diesel::update(comment_reply.find(comment_reply_id))
      .set(comment_reply_form)
      .get_result::<Self>(conn)
  }
}

impl CommentReply {
  pub fn mark_all_as_read(
    conn: &mut PgConnection,
    for_recipient_id: PersonId,
  ) -> Result<Vec<CommentReply>, Error> {
    use crate::schema::comment_reply::dsl::*;
    diesel::update(
      comment_reply
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
  }

  pub fn read_by_comment(
    conn: &mut PgConnection,
    for_comment_id: CommentId,
  ) -> Result<Self, Error> {
    use crate::schema::comment_reply::dsl::*;
    comment_reply
      .filter(comment_id.eq(for_comment_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      comment::*,
      comment_reply::*,
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::*,
      post::*,
    },
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let inserted_instance = Instance::create(conn, "my_domain.tld").unwrap();

    let new_person = PersonInsertForm::builder()
      .name("terrylake".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let recipient_form = PersonInsertForm::builder()
      .name("terrylakes recipient".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(conn, &recipient_form).unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community lake".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(conn, &new_post).unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(conn, &comment_form, None).unwrap();

    let comment_reply_form = CommentReplyInsertForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_reply = CommentReply::create(conn, &comment_reply_form).unwrap();

    let expected_reply = CommentReply {
      id: inserted_reply.id,
      recipient_id: inserted_reply.recipient_id,
      comment_id: inserted_reply.comment_id,
      read: false,
      published: inserted_reply.published,
    };

    let read_reply = CommentReply::read(conn, inserted_reply.id).unwrap();

    let comment_reply_update_form = CommentReplyUpdateForm { read: Some(false) };
    let updated_reply =
      CommentReply::update(conn, inserted_reply.id, &comment_reply_update_form).unwrap();

    Comment::delete(conn, inserted_comment.id).unwrap();
    Post::delete(conn, inserted_post.id).unwrap();
    Community::delete(conn, inserted_community.id).unwrap();
    Person::delete(conn, inserted_person.id).unwrap();
    Person::delete(conn, inserted_recipient.id).unwrap();
    Instance::delete(conn, inserted_instance.id).unwrap();

    assert_eq!(expected_reply, read_reply);
    assert_eq!(expected_reply, inserted_reply);
    assert_eq!(expected_reply, updated_reply);
  }
}
