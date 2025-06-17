use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    community::CommunityActions,
    instance::InstanceActions,
    keyword_block::LocalUserKeywordBlock,
    person::PersonActions,
  },
  traits::Blockable,
};
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_inbox_combined::impls::InboxCombinedQuery;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_content_combined::impls::PersonContentCombinedQuery;
use lemmy_db_views_person_liked_combined::impls::PersonLikedCombinedQuery;
use lemmy_db_views_person_saved_combined::impls::PersonSavedCombinedQuery;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::api::ExportDataResponse;
use lemmy_utils::{self, error::LemmyResult};

pub async fn export_data(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ExportDataResponse>> {
  let local_user_id = local_user_view.local_user.id;
  let local_instance_id = local_user_view.person.instance_id;
  let my_person_id = local_user_view.person.id;
  let my_person = &local_user_view.person;
  let local_user = &local_user_view.local_user;

  let pool = &mut context.pool();

  // TODO need to figure out how to override limits, and also what the max should be
  // let limit =

  let content = PersonContentCombinedQuery::new(my_person_id)
    .list(pool, Some(&local_user_view), local_instance_id)
    .await?;

  let liked = PersonLikedCombinedQuery::default()
    .list(pool, &local_user_view)
    .await?;

  let saved = PersonSavedCombinedQuery::default()
    .list(pool, &local_user_view)
    .await?;

  let read_posts = PostView::list_read(&mut context.pool(), my_person, None, None, None).await?;

  let hidden_posts =
    PostView::list_hidden(&mut context.pool(), my_person, None, None, None).await?;

  let inbox = InboxCombinedQuery::default()
    .list(&mut context.pool(), my_person_id, local_instance_id)
    .await?;

  let follows = CommunityFollowerView::for_person(pool, my_person_id).await?;

  let community_blocks = CommunityActions::read_blocks_for_person(pool, my_person_id).await?;

  let instance_blocks = InstanceActions::read_blocks_for_person(pool, my_person_id).await?;

  let person_blocks = PersonActions::read_blocks_for_person(pool, my_person_id).await?;

  let moderates =
    CommunityModeratorView::for_person(&mut context.pool(), my_person_id, Some(local_user)).await?;

  let keyword_blocks = LocalUserKeywordBlock::read(pool, local_user_id).await?;

  let discussion_languages = LocalUserLanguage::read(pool, local_user_id).await?;

  Ok(Json(ExportDataResponse {
    local_user_view,
    follows,
    moderates,
    community_blocks,
    instance_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
    inbox,
    content,
    liked,
    saved,
    read_posts,
    hidden_posts,
  }))
}
