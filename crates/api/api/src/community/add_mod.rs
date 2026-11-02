use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::AsyncConnection;
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::source::{
  community::{Community, CommunityActions, CommunityModeratorForm},
  local_user::LocalUser,
  modlog::{Modlog, ModlogInsertForm},
  person::Person,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::api::{AddModToCommunity, AddModToCommunityResponse};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, traits::Crud};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};

pub async fn add_mod_to_community(
  Json(data): Json<AddModToCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddModToCommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  // Verify that only mods or admins can add mod
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // Check that you're a higher mod.
  // Do this even if you're adding a mod (so that lower mods can't re-order higher ones)
  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    community.id,
    local_user_view.person.id,
    vec![data.person_id],
  )
  .await?;

  if !data.added {
    // Dont allow the last community mod to remove himself
    let mods = CommunityModeratorView::for_community(&mut context.pool(), community.id).await?;
    if !local_user_view.local_user.admin && mods.len() == 1 {
      return Err(LemmyErrorType::CannotLeaveMod.into());
    }
  }

  // If user is admin and community is remote, explicitly check that he is a
  // moderator. This is necessary because otherwise the action would be rejected
  // by the community's home instance.
  if local_user_view.local_user.admin && !community.local {
    CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      community.id,
      local_user_view.person.id,
    )
    .await?;
  }

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  let (action, accept_follow) = conn
    .transaction::<_, LemmyError, _>(async |conn| {
      let mut accept_follow = None;
      // Update in local database
      let community_moderator_form =
        CommunityModeratorForm::new(tx_data.community_id, tx_data.person_id);
      if tx_data.added {
        CommunityActions::join(&mut conn.into(), &community_moderator_form).await?;

        // Mods can view private communities without approval, so a pending follow request from a
        // user who becomes mod is accepted right away. Otherwise it would be counted but never
        // listed in the pending follows. A remote user's instance learns about it from the Accept.
        if community.local {
          let new_mod = Person::read(&mut conn.into(), tx_data.person_id).await?;
          let actions =
            CommunityActions::read(&mut conn.into(), tx_data.community_id, tx_data.person_id)
              .await?;
          if actions.follow_state == Some(CommunityFollowerState::ApprovalRequired) {
            CommunityActions::approve_private_community_follower(
              &mut conn.into(),
              tx_data.community_id,
              tx_data.person_id,
              local_user_view.person.id,
              CommunityFollowerState::Accepted,
            )
            .await?;
            if !new_mod.local {
              accept_follow = Some(SendActivityData::PrivateCommunityAcceptFollower {
                community_id: tx_data.community_id,
                person_id: tx_data.person_id,
                follow_activity_id: actions.follow_activity_id,
              });
            }
          }
        }
      } else {
        CommunityActions::leave(&mut conn.into(), &community_moderator_form).await?;
      }

      // Mod tables
      let form = ModlogInsertForm::mod_add_to_community(
        local_user_view.person.id,
        tx_data.community_id,
        tx_data.person_id,
        !tx_data.added,
      );
      let action = Modlog::create(&mut conn.into(), &[form]).await?;
      Ok((action, accept_follow))
    })
    .await?;
  notify_mod_action(action.clone(), &context);

  // Note: in case a remote mod is added, this returns the old moderators list, it will only get
  //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
  let community_id = data.community_id;
  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::AddModToCommunity {
      moderator: local_user_view.person,
      community_id: data.community_id,
      target: data.person_id,
      added: data.added,
    },
    &context,
  )?;
  if let Some(accept_follow) = accept_follow {
    ActivityChannel::submit_activity(accept_follow, &context)?;
  }

  Ok(Json(AddModToCommunityResponse { moderators }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      local_site::LocalSite,
      local_user::LocalUserInsertForm,
      person::PersonInsertForm,
    },
    test_data::TestData,
    traits::Followable,
  };
  use lemmy_db_schema_file::enums::CommunityVisibility;
  use lemmy_db_views_community_follower_approval::PendingFollowerView;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_add_mod_accepts_pending_follow() -> LemmyResult<()> {
    check_add_mod_accepts_pending_follow(true).await
  }

  #[tokio::test]
  #[serial]
  async fn test_add_mod_accepts_remote_pending_follow() -> LemmyResult<()> {
    check_add_mod_accepts_pending_follow(false).await
  }

  async fn check_add_mod_accepts_pending_follow(local_follower: bool) -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = TestData::create(pool).await?;

    let mod_person =
      Person::create(pool, &PersonInsertForm::test_form(data.instance.id, "mod")).await?;
    LocalUser::create(pool, &LocalUserInsertForm::test_form(mod_person.id), vec![]).await?;
    let mod_view = LocalUserView::read_person(pool, mod_person.id).await?;

    let follower_instance_id = if local_follower {
      data.instance.id
    } else {
      Instance::read_or_create(pool, "remote.tld").await?.id
    };
    let follower_form = PersonInsertForm {
      local: Some(local_follower),
      ..PersonInsertForm::test_form(follower_instance_id, "follower")
    };
    let follower = Person::create(pool, &follower_form).await?;
    if local_follower {
      LocalUser::create(pool, &LocalUserInsertForm::test_form(follower.id), vec![]).await?;
    }

    let community_form = CommunityInsertForm {
      visibility: Some(CommunityVisibility::Private),
      ..CommunityInsertForm::new(
        data.instance.id,
        "private_community".to_string(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;
    CommunityActions::join(
      pool,
      &CommunityModeratorForm::new(community.id, mod_person.id),
    )
    .await?;

    // The follow request is made before the follower is appointed as mod
    let follower_form = CommunityFollowerForm::new(
      community.id,
      follower.id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &follower_form).await?;
    assert_eq!(
      1,
      PendingFollowerView::count_approval_required(pool, mod_person.id).await?
    );

    add_mod_to_community(
      Json(AddModToCommunity {
        community_id: community.id,
        person_id: follower.id,
        added: true,
      }),
      context.clone(),
      mod_view,
    )
    .await?;

    let actions = CommunityActions::read(pool, community.id, follower.id).await?;
    assert_eq!(Some(CommunityFollowerState::Accepted), actions.follow_state);
    assert_eq!(Some(mod_person.id), actions.follow_approver_id);
    assert!(actions.became_moderator_at.is_some());
    assert_eq!(
      0,
      PendingFollowerView::count_approval_required(pool, mod_person.id).await?
    );

    LocalSite::delete(pool).await?;
    if !local_follower {
      Instance::delete(pool, follower_instance_id).await?;
    }
    data.delete(pool).await?;

    Ok(())
  }
}
