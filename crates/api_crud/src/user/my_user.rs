use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, site::MyUserInfo, utils::check_user_valid};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    community::CommunityActions,
    instance::InstanceActions,
    person::PersonActions,
  },
  traits::Blockable,
};
use lemmy_db_views::structs::{CommunityFollowerView, CommunityModeratorView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn get_my_user(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<MyUserInfo>> {
  check_user_valid(&local_user_view.person)?;

  // Build the local user with parallel queries and add it to site response
  let person_id = local_user_view.person.id;
  let local_user_id = local_user_view.local_user.id;
  let pool = &mut context.pool();

  // TODO this try join isn't working with LemmyError, just read them
  // let (follows, community_blocks, instance_blocks, person_blocks, moderates,
  // discussion_languages) =   lemmy_db_schema::try_join_with_pool!(pool => (
  //     |pool| CommunityFollowerView::for_person(pool, person_id),
  //     |pool| CommunityActions::read_blocks_for_person(pool, person_id),
  //     |pool| InstanceActions::read_blocks_for_person(pool, person_id),
  //     |pool| PersonActions::read_blocks_for_person(pool, person_id),
  //     |pool| CommunityModeratorView::for_person(pool, person_id, Some(&local_user_view.local_user)),
  //     |pool| LocalUserLanguage::read(pool, local_user_id)
  //   ))
  //   .with_lemmy_type(LemmyErrorType::SystemErrLogin)?;
  let follows = CommunityFollowerView::for_person(pool, person_id).await?;
  let community_blocks = CommunityActions::read_blocks_for_person(pool, person_id).await?;
  let instance_blocks = InstanceActions::read_blocks_for_person(pool, person_id).await?;
  let person_blocks = PersonActions::read_blocks_for_person(pool, person_id).await?;
  let moderates =
    CommunityModeratorView::for_person(pool, person_id, Some(&local_user_view.local_user)).await?;
  let discussion_languages = LocalUserLanguage::read(pool, local_user_id).await?;

  Ok(Json(MyUserInfo {
    local_user_view: local_user_view.clone(),
    follows,
    moderates,
    community_blocks,
    instance_blocks,
    person_blocks,
    discussion_languages,
  }))
}
