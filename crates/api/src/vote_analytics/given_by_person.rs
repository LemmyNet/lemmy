use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use chrono::{TimeZone, Utc};
use lemmy_api_common::{context::LemmyContext, person::GetVoteAnalyticsByPerson, utils::is_admin};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::VoteAnalyticsGivenByPersonView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn get_vote_analytics_given_by_person(
  data: Query<GetVoteAnalyticsByPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<VoteAnalyticsGivenByPersonView>> {
  is_admin(&local_user_view)?;

  let since = match data.start_time {
    Some(t) => Some(
      Utc
        .timestamp_opt(t, 0)
        .single()
        .ok_or(LemmyErrorType::InvalidUnixTime)?,
    ),
    _ => None,
  };
  let until = match data.end_time {
    Some(t) => Some(
      Utc
        .timestamp_opt(t, 0)
        .single()
        .ok_or(LemmyErrorType::InvalidUnixTime)?,
    ),
    _ => None,
  };

  let view = VoteAnalyticsGivenByPersonView::read(
    &mut context.pool(),
    data.person_id,
    since,
    until,
    data.limit,
  )
  .await?;

  Ok(Json(view))
}
