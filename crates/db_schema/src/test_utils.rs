#![allow(clippy::unwrap_used)]

use crate::{
  source::{
    comment::{Comment, CommentInsertForm},
    community::{Community, CommunityInsertForm},
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
  },
  traits::Crud,
  utils::DbPool,
};

#[async_trait]
pub trait TestDao {
  fn pool(&self) -> DbPool;

  async fn create_user(&self, instance: &Instance, name: &str) -> (Person, LocalUser) {
    let pool = &mut self.pool();
    let test_person_form = PersonInsertForm::builder()
      .name(name.to_string())
      .admin(Some(true))
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let test_person = Person::create(pool, &test_person_form).await.unwrap();

    let test_local_user_form = LocalUserInsertForm::builder()
      .person_id(test_person.id)
      .password_encrypted("test_password".to_string())
      .email(Some(format!("{}@domain.com", name)))
      .build();

    let test_local_user = LocalUser::create(pool, &test_local_user_form)
      .await
      .unwrap();

    (test_person, test_local_user)
  }

  async fn update_user(
    &self,
    local_user: &LocalUser,
    update_form: &LocalUserUpdateForm,
  ) -> LocalUser {
    let pool = &mut self.pool();

    LocalUser::update(pool, local_user.id, update_form)
      .await
      .unwrap()
  }

  async fn create_community(&self, instance: &Instance, name: &str) -> Community {
    let pool = &mut self.pool();
    let community_insertion_form = CommunityInsertForm::builder()
      .instance_id(instance.id)
      .name(name.to_string())
      .title(name.to_string())
      .build();

    Community::create(pool, &community_insertion_form)
      .await
      .unwrap()
  }

  async fn create_post(&self, poster: &Person, community: &Community, name: &str) -> Post {
    let pool = &mut self.pool();
    let post_insert_form = PostInsertForm::builder()
      .name(name.to_string())
      .creator_id(poster.id)
      .community_id(community.id)
      .build();

    Post::create(pool, &post_insert_form).await.unwrap()
  }

  async fn create_comment(
    &self,
    commenter: &Person,
    post: &Post,
    content: &str,
    parent: Option<&Comment>,
  ) -> Comment {
    let pool = &mut self.pool();
    let comment_insert_form = CommentInsertForm::builder()
      .creator_id(commenter.id)
      .post_id(post.id)
      .content(content.to_string())
      .build();
    let parent_path = parent.map(|parent_comment| parent_comment.path.clone());

    Comment::create(pool, &comment_insert_form, parent_path.as_ref())
      .await
      .unwrap()
  }
}
