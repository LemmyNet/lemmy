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
use lemmy_db_views_inbox_combined::{impls::InboxCombinedQuery, InboxCombinedView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_content_combined::{
  impls::PersonContentCombinedQuery,
  PersonContentCombinedView,
};
use lemmy_db_views_person_liked_combined::{
  impls::PersonLikedCombinedQuery,
  PersonLikedCombinedView,
};
use lemmy_db_views_person_saved_combined::{
  impls::PersonSavedCombinedQuery,
  PersonSavedCombinedView,
};
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::api::{ExportDataResponse, PostOrCommentOrPrivateMessage};
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

  let content = PersonContentCombinedQuery {
    no_limit: Some(true),
    ..PersonContentCombinedQuery::new(my_person_id)
  }
  .list(pool, Some(&local_user_view), local_instance_id)
  .await?
  .into_iter()
  .map(|u| match u {
    PersonContentCombinedView::Post(pv) => PostOrCommentOrPrivateMessage::Post(pv.post),
    PersonContentCombinedView::Comment(cv) => PostOrCommentOrPrivateMessage::Comment(cv.comment),
  })
  .collect();

  let saved = PersonSavedCombinedQuery {
    no_limit: Some(true),
    ..PersonSavedCombinedQuery::default()
  }
  .list(pool, &local_user_view)
  .await?
  .into_iter()
  .map(|u| match u {
    PersonSavedCombinedView::Post(pv) => PostOrCommentOrPrivateMessage::Post(pv.post),
    PersonSavedCombinedView::Comment(cv) => PostOrCommentOrPrivateMessage::Comment(cv.comment),
  })
  .collect();

  let inbox = InboxCombinedQuery {
    no_limit: Some(true),
    ..InboxCombinedQuery::default()
  }
  .list(&mut context.pool(), my_person_id, local_instance_id)
  .await?
  .into_iter()
  .map(|u| match u {
    InboxCombinedView::CommentReply(cr) => PostOrCommentOrPrivateMessage::Comment(cr.comment),
    InboxCombinedView::CommentMention(cm) => PostOrCommentOrPrivateMessage::Comment(cm.comment),
    InboxCombinedView::PostMention(pm) => PostOrCommentOrPrivateMessage::Post(pm.post),
    InboxCombinedView::PrivateMessage(pm) => {
      PostOrCommentOrPrivateMessage::PrivateMessage(pm.private_message)
    }
  })
  .collect();

  let liked = PersonLikedCombinedQuery {
    no_limit: Some(true),
    ..PersonLikedCombinedQuery::default()
  }
  .list(pool, &local_user_view)
  .await?
  .into_iter()
  .map(|u| {
    match u {
      PersonLikedCombinedView::Post(pv) => pv.post.ap_id,
      PersonLikedCombinedView::Comment(cv) => cv.comment.ap_id,
    }
    .into()
  })
  .collect();

  let read_posts =
    PostView::list_read(&mut context.pool(), my_person, None, None, None, Some(true))
      .await?
      .into_iter()
      .map(|pv| pv.post.ap_id.into())
      .collect();

  let follows = CommunityFollowerView::for_person(pool, my_person_id)
    .await?
    .into_iter()
    .map(|cv| cv.community.ap_id.into())
    .collect();

  let moderates =
    CommunityModeratorView::for_person(&mut context.pool(), my_person_id, Some(local_user))
      .await?
      .into_iter()
      .map(|cv| cv.community.ap_id.into())
      .collect();

  let community_blocks = CommunityActions::read_blocks_for_person(pool, my_person_id)
    .await?
    .into_iter()
    .map(|c| c.ap_id.into())
    .collect();

  let instance_blocks = InstanceActions::read_blocks_for_person(pool, my_person_id)
    .await?
    .into_iter()
    .map(|i| i.domain)
    .collect();

  let person_blocks = PersonActions::read_blocks_for_person(pool, my_person_id)
    .await?
    .into_iter()
    .map(|p| p.ap_id.into())
    .collect();

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
  }))
}
