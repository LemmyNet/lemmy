use crate::objects::{
  comment::ApubComment,
  community::ApubCommunity,
  person::ApubPerson,
  post::ApubPost,
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use actix_web::web::Json;
use futures::{future::try_join_all, StreamExt};
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    comment::{CommentSaved, CommentSavedForm},
    community::{CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
    person_block::{PersonBlock, PersonBlockForm},
    post::{PostSaved, PostSavedForm},
  },
  traits::{Blockable, Crud, Followable, Saveable},
  utils::diesel_option_overwrite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType, LemmyResult},
  spawn_try_task,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Maximum number of follow/block URLs which can be imported at once, to prevent server overloading.
/// To import a larger backup, split it into multiple parts.
///
/// TODO: having the user manually split files will very be confusing
const MAX_URL_IMPORT_COUNT: usize = 1000;

/// Backup of user data. This struct should never be changed so that the data can be used as a
/// long-term backup in case the instance goes down unexpectedly. All fields are optional to allow
/// importing partial backups.
///
/// This data should not be parsed by apps/clients, but directly downloaded as a file.
///
/// Be careful with any changes to this struct, to avoid breaking changes which could prevent
/// importing older backups.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
}

#[tracing::instrument(skip(context))]
pub async fn export_settings(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<Json<UserSettingsBackup>, LemmyError> {
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
    followed_communities: vec_into(lists.followed_communities),
    blocked_communities: vec_into(lists.blocked_communities),
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
) -> Result<Json<SuccessResponse>, LemmyError> {
  let display_name = diesel_option_overwrite(data.display_name.clone());
  let bio = diesel_option_overwrite(data.bio.clone());

  let person_form = PersonUpdateForm {
    display_name,
    bio,
    matrix_user_id: Some(data.matrix_id.clone()),
    bot_account: data.bot_account,
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &person_form).await?;

  let local_user_form = LocalUserUpdateForm {
    show_nsfw: data.settings.as_ref().map(|s| s.show_nsfw),
    theme: data.settings.as_ref().map(|s| s.theme.clone()),
    default_sort_type: data.settings.as_ref().map(|s| s.default_sort_type),
    default_listing_type: data.settings.as_ref().map(|s| s.default_listing_type),
    interface_language: data.settings.as_ref().map(|s| s.interface_language.clone()),
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
  .await?;

  let url_count = data.followed_communities.len()
    + data.blocked_communities.len()
    + data.blocked_users.len()
    + data.saved_posts.len()
    + data.saved_comments.len();
  if url_count > MAX_URL_IMPORT_COUNT {
    Err(LemmyErrorType::UserBackupTooLarge)?;
  }

  spawn_try_task(async move {
    const PARALLELISM: usize = 10;
    let person_id = local_user_view.person.id;

    // These tasks fetch objects from remote instances which might be down.
    // TODO: Would be nice if we could send a list of failed items with api response, but then
    //       the request would likely timeout.
    let mut failed_items = vec![];

    info!(
      "Starting settings backup for {}",
      local_user_view.person.name
    );

    futures::stream::iter(
      data
        .followed_communities
        .clone()
        .into_iter()
        // reset_request_count works like clone, and is necessary to avoid running into request limit
        .map(|f| (f, context.reset_request_count()))
        .map(|(followed, context)| async move {
          // need to reset outgoing request count to avoid running into limit
          let community = followed.dereference(&context).await?;
          let form = CommunityFollowerForm {
            person_id,
            community_id: community.id,
            pending: true,
          };
          CommunityFollower::follow(&mut context.pool(), &form).await?;
          LemmyResult::Ok(())
        }),
    )
    .buffer_unordered(PARALLELISM)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .enumerate()
    .for_each(|(i, r)| {
      if let Err(e) = r {
        failed_items.push(data.followed_communities.get(i).map(|u| u.inner().clone()));
        info!("Failed to import followed community: {e}");
      }
    });

    futures::stream::iter(
      data
        .saved_posts
        .clone()
        .into_iter()
        .map(|s| (s, context.reset_request_count()))
        .map(|(saved, context)| async move {
          let post = saved.dereference(&context).await?;
          let form = PostSavedForm {
            person_id,
            post_id: post.id,
          };
          PostSaved::save(&mut context.pool(), &form).await?;
          LemmyResult::Ok(())
        }),
    )
    .buffer_unordered(PARALLELISM)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .enumerate()
    .for_each(|(i, r)| {
      if let Err(e) = r {
        failed_items.push(data.followed_communities.get(i).map(|u| u.inner().clone()));
        info!("Failed to import saved post community: {e}");
      }
    });

    futures::stream::iter(
      data
        .saved_comments
        .clone()
        .into_iter()
        .map(|s| (s, context.reset_request_count()))
        .map(|(saved, context)| async move {
          let comment = saved.dereference(&context).await?;
          let form = CommentSavedForm {
            person_id,
            comment_id: comment.id,
          };
          CommentSaved::save(&mut context.pool(), &form).await?;
          LemmyResult::Ok(())
        }),
    )
    .buffer_unordered(PARALLELISM)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .enumerate()
    .for_each(|(i, r)| {
      if let Err(e) = r {
        failed_items.push(data.followed_communities.get(i).map(|u| u.inner().clone()));
        info!("Failed to import saved comment community: {e}");
      }
    });

    let failed_items: Vec<_> = failed_items.into_iter().flatten().collect();
    info!(
      "Finished settings backup for {}, failed items: {:#?}",
      local_user_view.person.name, failed_items
    );

    // These tasks don't connect to any remote instances but only insert directly in the database.
    // That means the only error condition are db connection failures, so no extra error handling is
    // needed.
    try_join_all(data.blocked_communities.iter().map(|blocked| async {
      // dont fetch unknown blocked objects from home server
      let community = blocked.dereference_local(&context).await?;
      let form = CommunityBlockForm {
        person_id,
        community_id: community.id,
      };
      CommunityBlock::block(&mut context.pool(), &form).await?;
      LemmyResult::Ok(())
    }))
    .await?;

    try_join_all(data.blocked_users.iter().map(|blocked| async {
      // dont fetch unknown blocked objects from home server
      let target = blocked.dereference_local(&context).await?;
      let form = PersonBlockForm {
        person_id,
        target_id: target.id,
      };
      PersonBlock::block(&mut context.pool(), &form).await?;
      LemmyResult::Ok(())
    }))
    .await?;
    Ok(())
  });

  Ok(Json(Default::default()))
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    api::user_settings_backup::{export_settings, import_settings},
    objects::tests::init_context,
  };
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
  use lemmy_db_views_actor::structs::CommunityFollowerView;
  use lemmy_utils::error::LemmyErrorType;
  use serial_test::serial;
  use std::time::Duration;
  use tokio::time::sleep;

  async fn create_user(
    name: String,
    bio: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LocalUserView {
    let instance = Instance::read_or_create(&mut context.pool(), "example.com".to_string())
      .await
      .unwrap();
    let person_form = PersonInsertForm::builder()
      .name(name.clone())
      .display_name(Some(name.clone()))
      .bio(bio)
      .public_key("asd".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(&mut context.pool(), &person_form)
      .await
      .unwrap();

    let user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("pass".to_string())
      .build();
    let local_user = LocalUser::create(&mut context.pool(), &user_form)
      .await
      .unwrap();

    LocalUserView::read(&mut context.pool(), local_user.id)
      .await
      .unwrap()
  }

  #[tokio::test]
  #[serial]
  async fn test_settings_export_import() {
    let context = init_context().await;

    let export_user = create_user("hanna".to_string(), Some("my bio".to_string()), &context).await;

    let community_form = CommunityInsertForm::builder()
      .name("testcom".to_string())
      .title("testcom".to_string())
      .instance_id(export_user.person.instance_id)
      .build();
    let community = Community::create(&mut context.pool(), &community_form)
      .await
      .unwrap();
    let follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: export_user.person.id,
      pending: false,
    };
    CommunityFollower::follow(&mut context.pool(), &follower_form)
      .await
      .unwrap();

    let backup = export_settings(export_user.clone(), context.reset_request_count())
      .await
      .unwrap();

    let import_user = create_user("charles".to_string(), None, &context).await;

    import_settings(backup, import_user.clone(), context.reset_request_count())
      .await
      .unwrap();
    let import_user_updated = LocalUserView::read(&mut context.pool(), import_user.local_user.id)
      .await
      .unwrap();

    // wait for background task to finish
    sleep(Duration::from_millis(100)).await;

    assert_eq!(
      export_user.person.display_name,
      import_user_updated.person.display_name
    );
    assert_eq!(export_user.person.bio, import_user_updated.person.bio);

    let follows = CommunityFollowerView::for_person(&mut context.pool(), import_user.person.id)
      .await
      .unwrap();
    assert_eq!(follows.len(), 1);
    assert_eq!(follows[0].community.actor_id, community.actor_id);

    LocalUser::delete(&mut context.pool(), export_user.local_user.id)
      .await
      .unwrap();
    LocalUser::delete(&mut context.pool(), import_user.local_user.id)
      .await
      .unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn disallow_large_backup() {
    let context = init_context().await;

    let export_user = create_user("hanna".to_string(), Some("my bio".to_string()), &context).await;

    let mut backup = export_settings(export_user.clone(), context.reset_request_count())
      .await
      .unwrap();

    for _ in 0..251 {
      backup
        .followed_communities
        .push("http://example.com".parse().unwrap());
      backup
        .blocked_communities
        .push("http://example2.com".parse().unwrap());
      backup
        .saved_posts
        .push("http://example3.com".parse().unwrap());
      backup
        .saved_comments
        .push("http://example4.com".parse().unwrap());
    }

    let import_user = create_user("charles".to_string(), None, &context).await;

    let imported =
      import_settings(backup, import_user.clone(), context.reset_request_count()).await;

    assert_eq!(
      imported.err().unwrap().error_type,
      LemmyErrorType::UserBackupTooLarge
    );

    LocalUser::delete(&mut context.pool(), export_user.local_user.id)
      .await
      .unwrap();
    LocalUser::delete(&mut context.pool(), import_user.local_user.id)
      .await
      .unwrap();
  }
}
