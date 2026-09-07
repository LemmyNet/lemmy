use lemmy_db_schema_file::{CommunityId, PersonId};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::DbPool;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn check_is_mod_or_admin(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  community_id: CommunityId,
) -> LemmyResult<()> {
  let is_mod = CommunityModeratorView::check_is_community_moderator(pool, community_id, person_id)
    .await
    .is_ok();
  let is_admin = LocalUserView::read_person(pool, person_id)
    .await
    .is_ok_and(|t| t.local_user.admin);

  if is_mod || is_admin {
    Ok(())
  } else {
    Err(LemmyErrorType::NotAModOrAdmin.into())
  }
}

pub mod functions;
pub mod markdown_links;
pub mod mentions;
pub mod protocol;
pub mod test;
