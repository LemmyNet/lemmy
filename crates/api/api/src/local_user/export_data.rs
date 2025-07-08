use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::local_user::LocalUser;
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
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::{
  api::{ExportDataResponse, PostOrCommentOrPrivateMessage},
  impls::user_backup_list_to_user_settings_backup,
};
use lemmy_utils::{self, error::LemmyResult};

pub async fn export_data(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ExportDataResponse>> {
  use PostOrCommentOrPrivateMessage::*;

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
    PersonContentCombinedView::Post(pv) => Post(pv.post),
    PersonContentCombinedView::Comment(cv) => Comment(cv.comment),
  })
  .collect();

  let inbox = InboxCombinedQuery {
    no_limit: Some(true),
    ..InboxCombinedQuery::default()
  }
  .list(pool, my_person_id, local_instance_id)
  .await?
  .into_iter()
  .map(|u| match u {
    InboxCombinedView::CommentReply(cr) => Comment(cr.comment),
    InboxCombinedView::CommentMention(cm) => Comment(cm.comment),
    InboxCombinedView::PostMention(pm) => Post(pm.post),
    InboxCombinedView::PrivateMessage(pm) => PrivateMessage(pm.private_message),
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

  let read_posts = PostView::list_read(pool, my_person, None, None, None, Some(true))
    .await?
    .into_iter()
    .map(|pv| pv.post.ap_id.into())
    .collect();

  let moderates = CommunityModeratorView::for_person(pool, my_person_id, Some(local_user))
    .await?
    .into_iter()
    .map(|cv| cv.community.ap_id.into())
    .collect();

  let lists = LocalUser::export_backup(pool, local_user_view.person.id).await?;
  let settings = user_backup_list_to_user_settings_backup(local_user_view, lists);

  Ok(Json(ExportDataResponse {
    inbox,
    content,
    liked,
    read_posts,
    moderates,
    settings,
  }))
}
