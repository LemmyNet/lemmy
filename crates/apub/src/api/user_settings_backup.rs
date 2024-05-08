use crate::objects::{
  comment::ApubComment,
  community::ApubCommunity,
  person::ApubPerson,
  post::ApubPost,
};use itertools::Itertools;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Object};
use actix_web::web::Json;
use futures::{future::try_join_all, StreamExt};
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    comment::{CommentSaved, CommentSavedForm},
    community::{CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
    instance::Instance,
    instance_block::{InstanceBlock, InstanceBlockForm},
    local_user::{LocalUser, LocalUserUpdateForm},
    local_user_vote_display_mode::{LocalUserVoteDisplayMode, LocalUserVoteDisplayModeUpdateForm},
    person::{Person, PersonUpdateForm},
    person_block::{PersonBlock, PersonBlockForm},
    post::{PostSaved, PostSavedForm},
  },
  traits::{Blockable, Crud, Followable, Saveable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult, MAX_API_PARAM_ELEMENTS},
  spawn_try_task,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing::info;

const PARALLELISM: usize = 10;

/// Backup of user data. This struct should never be changed so that the data can be used as a
/// long-term backup in case the instance goes down unexpectedly. All fields are optional to allow
/// importing partial backups.
///
/// This data should not be parsed by apps/clients, but directly downloaded as a file.
///
/// Be careful with any changes to this struct, to avoid breaking changes which could prevent
/// importing older backups.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserSettingsBackup {
  pub display_name: Option<String>,
  pub bio: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub matrix_id: Option<String>,
  pub bot_account: Option<bool>,
  // TODO: might be worth making a separate struct for settings backup, to avoid breakage in case
  //       fields are renamed, and to avoid storing unnecessary fields like person_id or email
  pub settings: Option<LocalUser>,
  pub vote_display_mode_settings: Option<LocalUserVoteDisplayMode>,
  #[serde(default)]
  pub followed_communities: Vec<ObjectId<ApubCommunity>>,
  #[serde(default)]
  pub saved_posts: Vec<ObjectId<ApubPost>>,
  #[serde(default)]
  pub saved_comments: Vec<ObjectId<ApubComment>>,
  #[serde(default)]
  pub blocked_communities: Vec<ObjectId<ApubCommunity>>,
  #[serde(default)]
  pub blocked_users: Vec<ObjectId<ApubPerson>>,
  #[serde(default)]
  pub blocked_instances: Vec<String>,
}

#[tracing::instrument(skip(context))]
pub async fn export_settings(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UserSettingsBackup>> {
  let lists = LocalUser::export_backup(&mut context.pool(), local_user_view.person.id).await?;

  let vec_into = |vec: Vec<_>| vec.into_iter().map(Into::into).collect();
  Ok(Json(UserSettingsBackup {
    display_name: local_user_view.person.display_name,
    bio: local_user_view.person.bio,
    avatar: local_user_view.person.avatar,
    banner: local_user_view.person.banner,
    matrix_id: local_user_view.person.matrix_user_id,
    bot_account: local_user_view.person.bot_account.into(),
    settings: Some(local_user_view.local_user),
    vote_display_mode_settings: Some(local_user_view.local_user_vote_display_mode),
    followed_communities: vec_into(lists.followed_communities),
    blocked_communities: vec_into(lists.blocked_communities),
    blocked_instances: lists.blocked_instances,
    blocked_users: lists.blocked_users.into_iter().map(Into::into).collect(),
    saved_posts: lists.saved_posts.into_iter().map(Into::into).collect(),
    saved_comments: lists.saved_comments.into_iter().map(Into::into).collect(),
  }))
}

#[tracing::instrument(skip(context))]
pub async fn import_settings(
  data: Json<UserSettingsBackup>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let person_form = PersonUpdateForm {
    display_name: Some(data.display_name.clone()),
    bio: Some(data.bio.clone()),
    matrix_user_id: Some(data.matrix_id.clone()),
    bot_account: data.bot_account,
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &person_form).await?;

  let local_user_form = LocalUserUpdateForm {
    show_nsfw: data.settings.as_ref().map(|s| s.show_nsfw),
    theme: data.settings.clone().map(|s| s.theme.clone()),
    default_sort_type: data.settings.as_ref().map(|s| s.default_sort_type),
    default_listing_type: data.settings.as_ref().map(|s| s.default_listing_type),
    interface_language: data.settings.clone().map(|s| s.interface_language),
    show_avatars: data.settings.as_ref().map(|s| s.show_avatars),
    send_notifications_to_email: data
      .settings
      .as_ref()
      .map(|s| s.send_notifications_to_email),
    show_scores: data.settings.as_ref().map(|s| s.show_scores),
    show_bot_accounts: data.settings.as_ref().map(|s| s.show_bot_accounts),
    show_read_posts: data.settings.as_ref().map(|s| s.show_read_posts),
    open_links_in_new_tab: data.settings.as_ref().map(|s| s.open_links_in_new_tab),
    blur_nsfw: data.settings.as_ref().map(|s| s.blur_nsfw),
    auto_expand: data.settings.as_ref().map(|s| s.auto_expand),
    infinite_scroll_enabled: data.settings.as_ref().map(|s| s.infinite_scroll_enabled),
    post_listing_mode: data.settings.as_ref().map(|s| s.post_listing_mode),
    ..Default::default()
  };
  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await
  .ok();

  // Update the vote display mode settings
  let vote_display_mode_form = LocalUserVoteDisplayModeUpdateForm {
    score: data.vote_display_mode_settings.as_ref().map(|s| s.score),
    upvotes: data.vote_display_mode_settings.as_ref().map(|s| s.upvotes),
    downvotes: data
      .vote_display_mode_settings
      .as_ref()
      .map(|s| s.downvotes),
    upvote_percentage: data
      .vote_display_mode_settings
      .as_ref()
      .map(|s| s.upvote_percentage),
  };

  LocalUserVoteDisplayMode::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &vote_display_mode_form,
  )
  .await?;

  let url_count = data.followed_communities.len()
    + data.blocked_communities.len()
    + data.blocked_users.len()
    + data.blocked_instances.len()
    + data.saved_posts.len()
    + data.saved_comments.len();
  if url_count > MAX_API_PARAM_ELEMENTS {
    Err(LemmyErrorType::TooManyItems)?;
  }

  spawn_try_task(async move {
    let person_id = local_user_view.person.id;

    info!(
      "Starting settings import for {}",
      local_user_view.person.name
    );

    let failed_followed_communities = fetch_and_import(
      data.followed_communities.clone(),
      &context,
      |(followed, context)| async move {
        let community = followed.dereference(&context).await?;
        let form = CommunityFollowerForm {
          person_id,
          community_id: community.id,
          pending: true,
        };
        CommunityFollower::follow(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_saved_posts = fetch_and_import(
      data.saved_posts.clone(),
      &context,
      |(saved, context)| async move {
        let post = saved.dereference(&context).await?;
        let form = PostSavedForm {
          person_id,
          post_id: post.id,
        };
        PostSaved::save(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_saved_comments = fetch_and_import(
      data.saved_comments.clone(),
      &context,
      |(saved, context)| async move {
        let comment = saved.dereference(&context).await?;
        let form = CommentSavedForm {
          person_id,
          comment_id: comment.id,
        };
        CommentSaved::save(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_community_blocks = fetch_and_import(
      data.blocked_communities.clone(),
      &context,
      |(blocked, context)| async move {
        let community = blocked.dereference(&context).await?;
        let form = CommunityBlockForm {
          person_id,
          community_id: community.id,
        };
        CommunityBlock::block(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    let failed_user_blocks = fetch_and_import(
      data.blocked_users.clone(),
      &context,
      |(blocked, context)| async move {
        let context = context.reset_request_count();
        let target = blocked.dereference(&context).await?;
          let form = PersonBlockForm {
            person_id,
            target_id: target.id,
          };
          PersonBlock::block(&mut context.pool(), &form).await?;
        LemmyResult::Ok(())
      },
    )
    .await?;

    try_join_all(data.blocked_instances.iter().map(|domain| async {
      let instance = Instance::read_or_create(&mut context.pool(), domain.clone()).await?;
      let form = InstanceBlockForm {
        person_id,
        instance_id: instance.id,
      };
      InstanceBlock::block(&mut context.pool(), &form).await?;
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
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::api::user_settings_backup::{export_settings, import_settings, UserSettingsBackup};
  use activitypub_federation::config::Data;
  use lemmy_api_common::context::LemmyContext;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityFollower, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::{Crud, Followable},
  };
  use lemmy_db_views::structs::LocalUserView;
  use lemmy_db_views_actor::structs::{CommunityFollowerView};
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use std::time::Duration;
  use tokio::time::sleep;

  async fn create_user(
    name: String,
    bio: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<LocalUserView> {
    let instance = Instance::read_or_create(&mut context.pool(), "example.com".to_string()).await?;
    let person_form = PersonInsertForm::builder()
      .name(name.clone())
      .display_name(Some(name.clone()))
      .bio(bio)
      .public_key("asd".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(&mut context.pool(), &person_form).await?;

    let user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("pass".to_string())
      .build();
    let local_user = LocalUser::create(&mut context.pool(), &user_form, vec![]).await?;

    Ok(
      LocalUserView::read(&mut context.pool(), local_user.id)
        .await?
        .ok_or(LemmyErrorType::CouldntFindLocalUser)?,
    )
  }

  #[tokio::test]
  #[serial]
  async fn test_settings_export_import() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    let export_user =
      create_user("hanna".to_string(), Some("my bio".to_string()), &context).await?;

    let community_form = CommunityInsertForm::builder()
      .name("testcom".to_string())
      .title("testcom".to_string())
      .instance_id(export_user.person.instance_id)
      .build();
    let community = Community::create(&mut context.pool(), &community_form).await?;
    let follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: export_user.person.id,
      pending: false,
    };
    CommunityFollower::follow(&mut context.pool(), &follower_form).await?;

    let backup = export_settings(export_user.clone(), context.reset_request_count()).await?;

    let import_user = create_user("charles".to_string(), None, &context).await?;

    import_settings(backup, import_user.clone(), context.reset_request_count()).await?;

    // wait for background task to finish
    sleep(Duration::from_millis(1000)).await;

    let import_user_updated = LocalUserView::read(&mut context.pool(), import_user.local_user.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindLocalUser)?;

    assert_eq!(
      export_user.person.display_name,
      import_user_updated.person.display_name
    );
    assert_eq!(export_user.person.bio, import_user_updated.person.bio);

    let follows =
      CommunityFollowerView::for_person(&mut context.pool(), import_user.person.id).await?;
    assert_eq!(follows.len(), 1);
    assert_eq!(follows[0].community.actor_id, community.actor_id);

    LocalUser::delete(&mut context.pool(), export_user.local_user.id).await?;
    LocalUser::delete(&mut context.pool(), import_user.local_user.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_settings_partial_import() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    let export_user =
      create_user("hanna".to_string(), Some("my bio".to_string()), &context).await?;

    let community_form = CommunityInsertForm::builder()
      .name("testcom".to_string())
      .title("testcom".to_string())
      .instance_id(export_user.person.instance_id)
      .build();
    let community = Community::create(&mut context.pool(), &community_form).await?;
    let follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: export_user.person.id,
      pending: false,
    };
    CommunityFollower::follow(&mut context.pool(), &follower_form).await?;

    let backup = export_settings(export_user.clone(), context.reset_request_count()).await?;

    let import_user = create_user("charles".to_string(), None, &context).await?;

    let backup2 = UserSettingsBackup {
      followed_communities: backup.followed_communities.clone(),
      ..Default::default()
    };
    import_settings(
      actix_web::web::Json(backup2),
      import_user.clone(),
      context.reset_request_count(),
    )
    .await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn disallow_large_backup() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    let export_user =
      create_user("hanna".to_string(), Some("my bio".to_string()), &context).await?;

    let mut backup = export_settings(export_user.clone(), context.reset_request_count()).await?;

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

    let import_user = create_user("charles".to_string(), None, &context).await?;

    let imported =
      import_settings(backup, import_user.clone(), context.reset_request_count()).await;

    assert_eq!(
      imported.err().map(|e| e.error_type),
      Some(LemmyErrorType::TooManyItems)
    );

    LocalUser::delete(&mut context.pool(), export_user.local_user.id).await?;
    LocalUser::delete(&mut context.pool(), import_user.local_user.id).await?;
    Ok(())
  }
}
