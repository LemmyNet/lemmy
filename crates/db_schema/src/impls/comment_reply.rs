use crate::{
  newtypes::{CommentId, CommentReplyId, PersonId},
  schema::comment_reply::dsl::{comment_id, comment_reply, read, recipient_id},
  source::comment_reply::{CommentReply, CommentReplyInsertForm, CommentReplyUpdateForm},
  traits::Crud,
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for CommentReply {
  type InsertForm = CommentReplyInsertForm;
  type UpdateForm = CommentReplyUpdateForm;
  type IdType = CommentReplyId;
  async fn read(mut conn: impl DbConn, comment_reply_id: CommentReplyId) -> Result<Self, Error> {
    comment_reply
      .find(comment_reply_id)
      .first::<Self>(conn)
      .await
  }

  async fn create(
    mut conn: impl DbConn,
    comment_reply_form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesnt return the existing row here
    insert_into(comment_reply)
      .values(comment_reply_form)
      .on_conflict((recipient_id, comment_id))
      .do_update()
      .set(comment_reply_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    mut conn: impl DbConn,
    comment_reply_id: CommentReplyId,
    comment_reply_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(comment_reply.find(comment_reply_id))
      .set(comment_reply_form)
      .get_result::<Self>(conn)
      .await
  }
}

impl CommentReply {
  pub async fn mark_all_as_read(
    mut conn: impl DbConn,
    for_recipient_id: PersonId,
  ) -> Result<Vec<CommentReply>, Error> {
    diesel::update(
      comment_reply
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn read_by_comment(
    mut conn: impl DbConn,
    for_comment_id: CommentId,
  ) -> Result<Self, Error> {
    comment_reply
      .filter(comment_id.eq(for_comment_id))
      .first::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      comment_reply::{CommentReply, CommentReplyInsertForm, CommentReplyUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(&mut *conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("terrylake".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(&mut *conn, &new_person).await.unwrap();

    let recipient_form = PersonInsertForm::builder()
      .name("terrylakes recipient".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(&mut *conn, &recipient_form).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community lake".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(&mut *conn, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(&mut *conn, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(&mut *conn, &comment_form, None)
      .await
      .unwrap();

    let comment_reply_form = CommentReplyInsertForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_reply = CommentReply::create(&mut *conn, &comment_reply_form)
      .await
      .unwrap();

    let expected_reply = CommentReply {
      id: inserted_reply.id,
      recipient_id: inserted_reply.recipient_id,
      comment_id: inserted_reply.comment_id,
      read: false,
      published: inserted_reply.published,
    };

    let read_reply = CommentReply::read(&mut *conn, inserted_reply.id)
      .await
      .unwrap();

    let comment_reply_update_form = CommentReplyUpdateForm { read: Some(false) };
    let updated_reply =
      CommentReply::update(&mut *conn, inserted_reply.id, &comment_reply_update_form)
        .await
        .unwrap();

    Comment::delete(&mut *conn, inserted_comment.id)
      .await
      .unwrap();
    Post::delete(&mut *conn, inserted_post.id).await.unwrap();
    Community::delete(&mut *conn, inserted_community.id)
      .await
      .unwrap();
    Person::delete(&mut *conn, inserted_person.id)
      .await
      .unwrap();
    Person::delete(&mut *conn, inserted_recipient.id)
      .await
      .unwrap();
    Instance::delete(&mut *conn, inserted_instance.id)
      .await
      .unwrap();

    assert_eq!(expected_reply, read_reply);
    assert_eq!(expected_reply, inserted_reply);
    assert_eq!(expected_reply, updated_reply);
  }
}
