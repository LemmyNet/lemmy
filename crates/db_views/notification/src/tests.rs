use crate::{impls::NotificationQuery, NotificationData, NotificationView};
use lemmy_db_schema::{
  assert_length,
  source::{
    community::{Community, CommunityInsertForm},
    instance::Instance,
    mod_log::moderator::{ModRemovePost, ModRemovePostForm},
    notification::{Notification, NotificationInsertForm},
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
    private_message::{PrivateMessage, PrivateMessageInsertForm},
  },
  traits::Crud,
  utils::{build_db_pool_for_tests, DbPool},
};
use lemmy_db_schema_file::enums::NotificationType;
use lemmy_utils::error::LemmyResult;
use pretty_assertions::assert_eq;
use serial_test::serial;

struct Data {
  alice: Person,
  bob: Person,
}

async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
  let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

  let alice_form = PersonInsertForm::test_form(instance.id, "alice2");
  let alice = Person::create(pool, &alice_form).await?;

  let bob_form = PersonInsertForm::test_form(instance.id, "bob2");
  let bob = Person::create(pool, &bob_form).await?;

  Ok(Data { alice, bob })
}

async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
  Instance::delete(pool, data.bob.instance_id).await?;
  Ok(())
}

#[tokio::test]
#[serial]
async fn test_private_message() -> LemmyResult<()> {
  let pool = &build_db_pool_for_tests();
  let pool = &mut pool.into();
  let data = init_data(pool).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(0, notifs);

  let form = &PrivateMessageInsertForm::new(data.bob.id, data.alice.id, "my message".to_string());
  let pm = PrivateMessage::create(pool, form).await?;
  let form = NotificationInsertForm::new_private_message(&pm);
  Notification::create(pool, &[form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(1, notifs);
  assert_eq!(Some(pm.id), notifs[0].notification.private_message_id);
  assert_eq!(pm.recipient_id, notifs[0].notification.recipient_id);
  assert!(!notifs[0].notification.read);
  let NotificationData::PrivateMessage(notif_pm) = &notifs[0].data else {
    panic!();
  };
  assert_eq!(pm, notif_pm.private_message);

  cleanup(data, pool).await
}

#[tokio::test]
#[serial]
async fn test_post() -> LemmyResult<()> {
  let pool = &build_db_pool_for_tests();
  let pool = &mut pool.into();
  let data = init_data(pool).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(0, notifs);

  let community_form = CommunityInsertForm::new(
    data.alice.instance_id,
    "comm".to_string(),
    "title".to_string(),
    "pubkey".to_string(),
  );
  let community = Community::create(pool, &community_form).await?;

  let post_form = PostInsertForm::new("title".to_string(), data.bob.id, community.id);
  let post = Post::create(pool, &post_form).await?;

  let notif_form =
    NotificationInsertForm::new_post(post.id, data.alice.id, NotificationType::Subscribed);
  Notification::create(pool, &[notif_form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs1 = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(1, notifs1);
  assert_eq!(Some(post.id), notifs1[0].notification.post_id);
  assert!(!notifs1[0].notification.read);
  let NotificationData::Post(notif_post) = &notifs1[0].data else {
    panic!();
  };
  assert_eq!(post, notif_post.post);
  Notification::mark_read_by_id_and_person(pool, notifs1[0].notification.id, true, data.alice.id)
    .await?;
  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);

  // create a notification entry for removed post
  let mod_remove_post_form = ModRemovePostForm {
    mod_person_id: data.bob.id,
    post_id: post.id,
    reason: "reason".to_string(),
    removed: Some(true),
  };
  let mod_remove_post = ModRemovePost::create(pool, &mod_remove_post_form).await?;
  let notif_form = NotificationInsertForm {
    mod_remove_post_id: Some(mod_remove_post.id),
    ..NotificationInsertForm::new(data.alice.id, NotificationType::ModAction)
  };
  Notification::create(pool, &[notif_form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs2 = NotificationQuery {
    unread_only: Some(true),
    ..Default::default()
  }
  .list(pool, &data.alice)
  .await?;
  assert_length!(1, notifs2);
  assert_eq!(
    Some(mod_remove_post.id),
    notifs2[0].notification.mod_remove_post_id
  );
  assert!(!notifs2[0].notification.read);
  let NotificationData::ModRemovePost(notif_remove_post) = &notifs2[0].data else {
    panic!();
  };
  assert_eq!(&mod_remove_post, notif_remove_post);

  Notification::delete(pool, notifs1[0].notification.id).await?;
  Notification::delete(pool, notifs2[0].notification.id).await?;
  cleanup(data, pool).await
}
