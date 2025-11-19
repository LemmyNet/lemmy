use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::{NotificationData, impls::NotificationQuery};
use lemmy_db_views_person_content_combined::{
  PersonContentCombinedView,
  impls::PersonContentCombinedQuery,
};
use lemmy_db_views_person_liked_combined::{
  PersonLikedCombinedView,
  impls::PersonLikedCombinedQuery,
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

  let notifications = NotificationQuery {
    no_limit: Some(true),
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    ..NotificationQuery::default()
  }
  .list(pool, &local_user_view.person)
  .await?
  .into_iter()
  .flat_map(|u| match u.data {
    NotificationData::Post(p) => Some(Post(p.post)),
    NotificationData::Comment(c) => Some(Comment(c.comment)),
    NotificationData::PrivateMessage(pm) => Some(PrivateMessage(pm.private_message)),
    // skip modlog items
    NotificationData::ModAction(_) => None,
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

  let read_posts = PostView::list_read(pool, my_person, None, None, Some(true))
    .await?
    .into_iter()
    .map(|pv| pv.post.ap_id.into())
    .collect();

  let moderates = CommunityModeratorView::for_person(pool, my_person_id, Some(local_user))
    .await?
    .into_iter()
    .map(|cv| cv.community.ap_id.into())
    .collect();

  let settings =
    user_backup_list_to_user_settings_backup(local_user_view, &mut context.pool()).await?;

  Ok(Json(ExportDataResponse {
    notifications,
    content,
    liked,
    read_posts,
    moderates,
    settings,
  }))
}
