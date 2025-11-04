use crate::protocol::block::{block_user::BlockUser, undo_block_user::UndoBlockUser};
use activitypub_federation::{config::Data, kinds::public, traits::Object};
use either::Either;
use lemmy_api_utils::{context::LemmyContext, utils::check_expire_time};
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, instance::ApubSite},
  utils::functions::generate_to,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{comment::Comment, community::Community, person::Person, post::Post, site::Site},
};
use lemmy_db_views_community::api::BanFromCommunity;
use lemmy_diesel_utils::{connection::DbPool, traits::Crud};
use lemmy_utils::error::LemmyResult;
use url::Url;

pub mod block_user;
pub mod undo_block_user;

pub type SiteOrCommunity = Either<ApubSite, ApubCommunity>;

async fn generate_cc(target: &SiteOrCommunity, pool: &mut DbPool<'_>) -> LemmyResult<Vec<Url>> {
  Ok(match target {
    SiteOrCommunity::Left(_) => Site::read_remote_sites(pool)
      .await?
      .into_iter()
      .map(|s| s.ap_id.into())
      .collect(),
    SiteOrCommunity::Right(c) => vec![c.id().clone()],
  })
}

pub(crate) async fn send_ban_from_site(
  moderator: Person,
  banned_user: Person,
  reason: String,
  remove_or_restore_data: Option<bool>,
  ban: bool,
  expires: Option<i64>,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let site = SiteOrCommunity::Left(Site::read_local(&mut context.pool()).await?.into());
  let expires = check_expire_time(expires)?;

  if ban {
    BlockUser::send(
      &site,
      &banned_user.into(),
      &moderator.into(),
      remove_or_restore_data.unwrap_or(false),
      reason.clone(),
      expires,
      &context,
    )
    .await
  } else {
    UndoBlockUser::send(
      &site,
      &banned_user.into(),
      &moderator.into(),
      remove_or_restore_data.unwrap_or(false),
      reason.clone(),
      &context,
    )
    .await
  }
}

pub(crate) async fn send_ban_from_community(
  mod_: Person,
  community_id: CommunityId,
  banned_person: Person,
  data: BanFromCommunity,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
    .await?
    .into();
  let expires_at = check_expire_time(data.expires_at)?;

  if data.ban {
    BlockUser::send(
      &SiteOrCommunity::Right(community),
      &banned_person.into(),
      &mod_.into(),
      data.remove_or_restore_data.unwrap_or(false),
      data.reason.clone(),
      expires_at,
      &context,
    )
    .await
  } else {
    UndoBlockUser::send(
      &SiteOrCommunity::Right(community),
      &banned_person.into(),
      &mod_.into(),
      data.remove_or_restore_data.unwrap_or(false),
      data.reason.clone(),
      &context,
    )
    .await
  }
}

fn to(target: &SiteOrCommunity) -> LemmyResult<Vec<Url>> {
  Ok(if let SiteOrCommunity::Right(c) = target {
    generate_to(c)?
  } else {
    vec![public()]
  })
}

// user banned from remote instance, remove content only in communities from that
// instance
async fn update_removed_for_instance(
  blocked_person: &Person,
  site: &ApubSite,
  removed: bool,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  Post::update_removed_for_creator_and_instance(pool, blocked_person.id, site.instance_id, removed)
    .await?;
  Comment::update_removed_for_creator_and_instance(
    pool,
    blocked_person.id,
    site.instance_id,
    removed,
  )
  .await?;
  Ok(())
}
