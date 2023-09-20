use crate::objects::{community::ApubCommunity, person::ApubPerson};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use actix_web::web::Json;
use futures::future::try_join_all;
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
    person_block::{PersonBlock, PersonBlockForm},
  },
  traits::{Blockable, Crud, Followable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  spawn_try_task,
};
use serde::{Deserialize, Serialize};

/// Maximum number of follow/block URLs which can be imported at once, to prevent server overloading.
/// To import a larger backup, split it into multiple parts.
///
/// TODO: having the user manually split files will very be confusing
const MAX_URL_IMPORT_COUNT: usize = 200;

/// Backup of user data. This struct should never be changed so that the data can be used as a
/// long-term backup in case the instance goes down unexpectedly. All fields are optional to allow
/// importing partial backups.
///
/// This data should not be parsed by apps/clients, but directly downloaded as a file.
///
/// Be careful with any changes to this struct, to avoid breaking changes which could prevent
/// importing older backups.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBackup {
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
  pub blocked_communities: Vec<ObjectId<ApubCommunity>>,
  #[serde(default)]
  pub blocked_users: Vec<ObjectId<ApubPerson>>,
}

#[tracing::instrument(skip(context))]
pub async fn export_user_backup(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<Json<UserBackup>, LemmyError> {
  let lists = LocalUser::export_backup(&mut context.pool(), local_user_view.person.id).await?;

  Ok(Json(UserBackup {
    display_name: local_user_view.person.display_name,
    bio: local_user_view.person.bio,
    avatar: local_user_view.person.avatar,
    banner: local_user_view.person.banner,
    matrix_id: local_user_view.person.matrix_user_id,
    bot_account: local_user_view.person.bot_account.into(),
    settings: Some(local_user_view.local_user),
    followed_communities: lists
      .followed_communities
      .into_iter()
      .map(Into::into)
      .collect(),
    blocked_communities: lists
      .blocked_communities
      .into_iter()
      .map(Into::into)
      .collect(),
    blocked_users: lists.blocked_users.into_iter().map(Into::into).collect(),
  }))
}

#[tracing::instrument(skip(context))]
pub async fn import_user_backup(
  data: Json<UserBackup>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // TODO: sanitize data

  let person_form = PersonUpdateForm {
    display_name: Some(data.display_name.clone()),
    bio: Some(data.bio.clone()),
    // TODO: might want to reupload avatar and banner to local instance
    avatar: Some(data.avatar.clone()),
    banner: Some(data.banner.clone()),
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

  let url_count =
    data.followed_communities.len() + data.blocked_communities.len() + data.blocked_users.len();
  if url_count > MAX_URL_IMPORT_COUNT {
    todo!();
  }

  let task = async move {
    try_join_all(data.followed_communities.iter().map(|blocked| async {
      // need to reset outgoing request count to avoid running into limit
      let context = context.reset_request_count();
      let community = blocked.dereference(&context).await?;
      let form = CommunityFollowerForm {
        person_id: local_user_view.person.id,
        community_id: community.id,
        pending: true,
      };
      CommunityFollower::follow(&mut context.pool(), &form).await?;
      LemmyResult::Ok(())
    }))
    .await?;

    try_join_all(data.blocked_communities.iter().map(|blocked| async {
      // dont fetch unknown blocked objects from home server
      let community = blocked.dereference_local(&context).await?;
      let form = CommunityBlockForm {
        person_id: local_user_view.person.id,
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
        person_id: local_user_view.person.id,
        target_id: target.id,
      };
      PersonBlock::block(&mut context.pool(), &form).await?;
      LemmyResult::Ok(())
    }))
    .await?;
    Ok(())
  };
  spawn_try_task(task);

  Ok(Json(Default::default()))
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_user_backup() {
    // create user account

    // call export function

    // create second account

    // call import function on it

    // check that data is identical
  }
}
