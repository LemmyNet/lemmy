use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Object};
use actix_web::web::Json;
use futures::{future::try_join_all, StreamExt};
use itertools::Itertools;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{
  comment::ApubComment,
  community::ApubCommunity,
  person::ApubPerson,
  post::ApubPost,
};
use lemmy_db_schema::{
  source::{
    comment::{CommentActions, CommentSavedForm},
    community::{CommunityActions, CommunityBlockForm, CommunityFollowerForm},
    instance::{Instance, InstanceActions, InstanceCommunitiesBlockForm, InstancePersonsBlockForm},
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonActions, PersonBlockForm, PersonUpdateForm},
    post::{PostActions, PostSavedForm},
  },
  traits::{Blockable, Crud, Followable, Saveable},
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{SuccessResponse, UserSettingsBackup},
  impls::user_backup_list_to_user_settings_backup,
};
use lemmy_utils::{
  error::LemmyResult,
  spawn_try_task,
  utils::validation::check_api_elements_count,
};
use serde::Deserialize;
use std::future::Future;
use tracing::info;

const PARALLELISM: usize = 10;

pub async fn export_settings(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UserSettingsBackup>> {
  let lists = LocalUser::export_backup(&mut context.pool(), local_user_view.person.id).await?;
  let settings = user_backup_list_to_user_settings_backup(local_user_view, lists);

  Ok(Json(settings))
}

pub async fn import_settings(
  data: Json<UserSettingsBackup>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let person_form = PersonUpdateForm {
    display_name: data.display_name.clone().map(Some),
    bio: data.bio.clone().map(Some),
    matrix_user_id: data.matrix_id.clone().map(Some),
    bot_account: data.bot_account,
    ..Default::default()
  };
  // ignore error in case form is empty
  Person::update(&mut context.pool(), local_user_view.person.id, &person_form)
    .await
    .ok();

  let local_user_form = LocalUserUpdateForm {
    show_nsfw: data.settings.as_ref().map(|s| s.show_nsfw),
    theme: data.settings.clone().map(|s| s.theme.clone()),
    default_post_sort_type: data.settings.as_ref().map(|s| s.default_post_sort_type),
    default_comment_sort_type: data.settings.as_ref().map(|s| s.default_comment_sort_type),
    default_listing_type: data.settings.as_ref().map(|s| s.default_listing_type),
    interface_language: data.settings.clone().map(|s| s.interface_language),
    show_avatars: data.settings.as_ref().map(|s| s.show_avatars),
    send_notifications_to_email: data
      .settings
      .as_ref()
      .map(|s| s.send_notifications_to_email),
    show_bot_accounts: data.settings.as_ref().map(|s| s.show_bot_accounts),
    show_read_posts: data.settings.as_ref().map(|s| s.show_read_posts),
    open_links_in_new_tab: data.settings.as_ref().map(|s| s.open_links_in_new_tab),
    blur_nsfw: data.settings.as_ref().map(|s| s.blur_nsfw),
    infinite_scroll_enabled: data.settings.as_ref().map(|s| s.infinite_scroll_enabled),
    post_listing_mode: data.settings.as_ref().map(|s| s.post_listing_mode),
    show_score: data.settings.as_ref().map(|s| s.show_score),
    show_upvotes: data.settings.as_ref().map(|s| s.show_upvotes),
    show_downvotes: data.settings.as_ref().map(|s| s.show_downvotes),
    show_upvote_percentage: data.settings.as_ref().map(|s| s.show_upvote_percentage),
    ..Default::default()
  };
  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  let url_count = data.followed_communities.len()
    + data.blocked_communities.len()
    + data.blocked_users.len()
    + data.blocked_instances_communities.len()
    + data.blocked_instances_persons.len()
    + data.saved_posts.len()
    + data.saved_comments.len();
  check_api_elements_count(url_count)?;

  spawn_try_task(async move {
    let person_id = local_user_view.person.id;

    info!(
      "Starting settings import for {}",
      local_user_view.person.name
    );

    let failed_followed_communities = fetch_and_import(
      data
        .followed_communities
        .clone()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ObjectId<ApubCommunity>>>(),
      &context,
      |(followed, context)| async move {
        let community = followed.dereference(&context).await?;
        let form =
          CommunityFollowerForm::new(community.id, person_id, CommunityFollowerState::Pending);
        CommunityActions::follow(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_saved_posts = fetch_and_import(
      data
        .saved_posts
        .clone()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ObjectId<ApubPost>>>(),
      &context,
      |(saved, context)| async move {
        let post = saved.dereference(&context).await?;
        let form = PostSavedForm::new(post.id, person_id);
        PostActions::save(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_saved_comments = fetch_and_import(
      data
        .saved_comments
        .clone()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ObjectId<ApubComment>>>(),
      &context,
      |(saved, context)| async move {
        let comment = saved.dereference(&context).await?;
        let form = CommentSavedForm::new(person_id, comment.id);
        CommentActions::save(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_community_blocks = fetch_and_import(
      data
        .blocked_communities
        .clone()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ObjectId<ApubCommunity>>>(),
      &context,
      |(blocked, context)| async move {
        let community = blocked.dereference(&context).await?;
        let form = CommunityBlockForm::new(community.id, person_id);
        CommunityActions::block(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_user_blocks = fetch_and_import(
      data
        .blocked_users
        .clone()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ObjectId<ApubPerson>>>(),
      &context,
      |(blocked, context)| async move {
        let target = blocked.dereference(&context).await?;
        let form = PersonBlockForm::new(person_id, target.id);
        PersonActions::block(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    try_join_all(
      data
        .blocked_instances_communities
        .iter()
        .map(|domain| async {
          let instance = Instance::read_or_create(&mut context.pool(), domain.clone()).await?;
          let form = InstanceCommunitiesBlockForm::new(person_id, instance.id);
          InstanceActions::block_communities(&mut context.pool(), &form).await?;
          LemmyResult::Ok(())
        }),
    )
    .await?;

    try_join_all(data.blocked_instances_persons.iter().map(|domain| async {
      let instance = Instance::read_or_create(&mut context.pool(), domain.clone()).await?;
      let form = InstancePersonsBlockForm::new(person_id, instance.id);
      InstanceActions::block_persons(&mut context.pool(), &form).await?;
      LemmyResult::Ok(())
    }))
    .await?;

    info!("Settings import completed for {}, the following items failed: {failed_followed_communities}, {failed_saved_posts}, {failed_saved_comments}, {failed_community_blocks}, {failed_user_blocks}",
    local_user_view.person.name);

    Ok(())
  });

  Ok(Json(Default::default()))
}

async fn fetch_and_import<Kind, Fut>(
  objects: Vec<ObjectId<Kind>>,
  context: &Data<LemmyContext>,
  import_fn: impl FnMut((ObjectId<Kind>, Data<LemmyContext>)) -> Fut,
) -> LemmyResult<String>
where
  Kind: Object + Send + 'static,
  for<'de2> <Kind as Object>::Kind: Deserialize<'de2>,
  Fut: Future<Output = LemmyResult<()>>,
{
  let mut failed_items = vec![];
  futures::stream::iter(
    objects
      .clone()
      .into_iter()
      // need to reset outgoing request count to avoid running into limit
      .map(|s| (s, context.reset_request_count()))
      .map(import_fn),
  )
  .buffer_unordered(PARALLELISM)
  .collect::<Vec<_>>()
  .await
  .into_iter()
  .enumerate()
  .for_each(|(i, r): (usize, LemmyResult<()>)| {
    if r.is_err() {
      if let Some(object) = objects.get(i) {
        failed_items.push(object.inner().clone());
      }
    }
  });
  Ok(failed_items.into_iter().join(","))
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
pub(crate) mod tests {
  use super::*;
  use crate::api::user_settings_backup::{export_settings, import_settings};
  use actix_web::web::Json;
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityActions, CommunityFollowerForm, CommunityInsertForm},
      person::Person,
    },
    test_data::TestData,
    traits::{Crud, Followable},
  };
  use lemmy_db_views_community_follower::CommunityFollowerView;
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use serial_test::serial;
  use std::time::Duration;
  use tokio::time::sleep;

  #[tokio::test]
  #[serial]
  async fn test_settings_export_import() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = TestData::create(pool).await?;

    let export_user = LocalUserView::create_test_user(pool, "hanna", "my bio", false).await?;

    let community_form = CommunityInsertForm::new(
      export_user.person.instance_id,
      "testcom".to_string(),
      "testcom".to_string(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;
    let follower_form = CommunityFollowerForm::new(
      community.id,
      export_user.person.id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &follower_form).await?;

    let backup = export_settings(export_user.clone(), context.clone()).await?;

    let import_user =
      LocalUserView::create_test_user(pool, "charles", "charles bio", false).await?;

    import_settings(backup, import_user.clone(), context.clone()).await?;

    // wait for background task to finish
    sleep(Duration::from_millis(1000)).await;

    let import_user_updated = LocalUserView::read(pool, import_user.local_user.id).await?;

    assert_eq!(
      export_user.person.display_name,
      import_user_updated.person.display_name
    );
    assert_eq!(export_user.person.bio, import_user_updated.person.bio);

    let follows = CommunityFollowerView::for_person(pool, import_user.person.id).await?;
    assert_eq!(follows.len(), 1);
    assert_eq!(follows[0].community.ap_id, community.ap_id);

    Person::delete(pool, export_user.person.id).await?;
    Person::delete(pool, import_user.person.id).await?;
    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn disallow_large_backup() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = TestData::create(pool).await?;

    let export_user = LocalUserView::create_test_user(pool, "harry", "harry bio", false).await?;

    let mut backup = export_settings(export_user.clone(), context.clone()).await?;

    for _ in 0..2501 {
      backup
        .followed_communities
        .push("http://example.com".parse()?);
      backup
        .blocked_communities
        .push("http://example2.com".parse()?);
      backup.saved_posts.push("http://example3.com".parse()?);
      backup.saved_comments.push("http://example4.com".parse()?);
    }

    let import_user = LocalUserView::create_test_user(pool, "sally", "sally bio", false).await?;

    let imported = import_settings(backup, import_user.clone(), context.clone()).await;

    assert_eq!(
      imported.err().map(|e| e.error_type),
      Some(LemmyErrorType::TooManyItems)
    );

    Person::delete(pool, export_user.person.id).await?;
    Person::delete(pool, import_user.person.id).await?;
    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn import_partial_backup() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = TestData::create(pool).await?;

    let import_user = LocalUserView::create_test_user(pool, "larry", "larry bio", false).await?;

    let backup =
      serde_json::from_str("{\"bot_account\": true, \"settings\": {\"theme\": \"my_theme\"}}")?;
    import_settings(Json(backup), import_user.clone(), context.clone()).await?;

    let import_user_updated = LocalUserView::read(pool, import_user.local_user.id).await?;
    // mark as bot account
    assert!(import_user_updated.person.bot_account);
    // dont remove existing bio
    assert_eq!(import_user.person.bio, import_user_updated.person.bio);
    // local_user can be deserialized without id/person_id fields
    assert_eq!("my_theme", import_user_updated.local_user.theme);

    data.delete(&mut context.pool()).await?;
    Ok(())
  }
}
