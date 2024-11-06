use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonCommentMentions, GetPersonCommentMentionsResponse},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::person_comment_mention_view::PersonCommentMentionQuery;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_comment_mentions(
  data: Query<GetPersonCommentMentions>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetPersonCommentMentionsResponse>> {
  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let unread_only = data.unread_only.unwrap_or_default();
  let person_id = Some(local_user_view.person.id);
  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;

  let comment_mentions = PersonCommentMentionQuery {
    recipient_id: person_id,
    my_person_id: person_id,
    sort,
    unread_only,
    show_bot_accounts,
    page,
    limit,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(GetPersonCommentMentionsResponse { comment_mentions }))
}
