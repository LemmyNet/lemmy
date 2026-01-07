use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_deleted};
use lemmy_db_schema::{
  MultiCommunityListingType,
  MultiCommunitySortType,
  source::{
    actor_language::LocalUserLanguage,
    community::CommunityActions,
    instance::InstanceActions,
    keyword_block::LocalUserKeywordBlock,
    person::PersonActions,
  },
  traits::Blockable,
};
use lemmy_db_views_community::impls::MultiCommunityQuery;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::MyUserInfo;
use lemmy_utils::error::LemmyResult;

pub async fn get_my_user(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<MyUserInfo>> {
  check_local_user_deleted(&local_user_view)?;

  // Build the local user with parallel queries and add it to site response
  let person_id = local_user_view.person.id;
  let local_user_id = local_user_view.local_user.id;
  let pool = &mut context.pool();

  let (
    follows,
    community_blocks,
    instance_communities_blocks,
    instance_persons_blocks,
    person_blocks,
    moderates,
    multi_community_follows,
    keyword_blocks,
    discussion_languages,
  ) = lemmy_diesel_utils::try_join_with_pool!(pool => (
    |pool| CommunityFollowerView::for_person(pool, person_id),
    |pool| CommunityActions::read_blocks_for_person(pool, person_id),
    |pool| InstanceActions::read_communities_block_for_person(pool, person_id),
    |pool| InstanceActions::read_persons_block_for_person(pool, person_id),
    |pool| PersonActions::read_blocks_for_person(pool, person_id),
    |pool| CommunityModeratorView::for_person(pool, person_id, Some(&local_user_view.local_user)),
    |pool| MultiCommunityQuery {
      my_person_id: Some(person_id),
      listing_type: Some(MultiCommunityListingType::Subscribed),
      sort: Some(MultiCommunitySortType::NameAsc),
      no_limit: Some(true),
      ..Default::default()
    }
    .list(pool),
    |pool| LocalUserKeywordBlock::read(pool, local_user_id),
    |pool| LocalUserLanguage::read(pool, local_user_id)
  ))?;

  Ok(Json(MyUserInfo {
    local_user_view: local_user_view.clone(),
    follows,
    moderates,
    multi_community_follows: multi_community_follows.items,
    community_blocks,
    instance_communities_blocks,
    instance_persons_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
  }))
}
