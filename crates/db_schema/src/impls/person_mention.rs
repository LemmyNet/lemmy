use crate::{
  newtypes::{CommentId, PersonId, PersonMentionId},
  schema::person_mention::dsl::{comment_id, person_mention, read, recipient_id},
  source::person_mention::{PersonMention, PersonMentionInsertForm, PersonMentionUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for PersonMention {
  type InsertForm = PersonMentionInsertForm;
  type UpdateForm = PersonMentionUpdateForm;
  type IdType = PersonMentionId;

  async fn create(
    pool: &mut DbPool<'_>,
    person_mention_form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesnt return the existing row here
    insert_into(person_mention)
      .values(person_mention_form)
      .on_conflict((recipient_id, comment_id))
      .do_update()
      .set(person_mention_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    person_mention_id: PersonMentionId,
    person_mention_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_mention.find(person_mention_id))
      .set(person_mention_form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PersonMention {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> Result<Vec<PersonMention>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_mention
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn read_by_comment_and_person(
    pool: &mut DbPool<'_>,
    for_comment_id: CommentId,
    for_recipient_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person_mention
      .filter(comment_id.eq(for_comment_id))
      .filter(recipient_id.eq(for_recipient_id))
      .first::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      person_mention::{PersonMention, PersonMentionInsertForm, PersonMentionUpdateForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("terrylake".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let recipient_form = PersonInsertForm::builder()
      .name("terrylakes recipient".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(pool, &recipient_form).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community lake".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    let person_mention_form = PersonMentionInsertForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_mention = PersonMention::create(pool, &person_mention_form)
      .await
      .unwrap();

    let expected_mention = PersonMention {
      id: inserted_mention.id,
      recipient_id: inserted_mention.recipient_id,
      comment_id: inserted_mention.comment_id,
      read: false,
      published: inserted_mention.published,
    };

    let read_mention = PersonMention::read(pool, inserted_mention.id)
      .await
      .unwrap();

    let person_mention_update_form = PersonMentionUpdateForm { read: Some(false) };
    let updated_mention =
      PersonMention::update(pool, inserted_mention.id, &person_mention_update_form)
        .await
        .unwrap();
    Comment::delete(pool, inserted_comment.id).await.unwrap();
    Post::delete(pool, inserted_post.id).await.unwrap();
    Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_person.id).await.unwrap();
    Person::delete(pool, inserted_recipient.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_mention, read_mention);
    assert_eq!(expected_mention, inserted_mention);
    assert_eq!(expected_mention, updated_mention);
  }
}
