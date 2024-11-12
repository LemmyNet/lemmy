use chrono::{DateTime, TimeZone, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::post::Post;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod delete;
pub mod read;
pub mod remove;
pub mod update;

async fn convert_published_time(
  scheduled_publish_time: Option<i64>,
  local_user_view: &LocalUserView,
  context: &LemmyContext,
) -> LemmyResult<Option<DateTime<Utc>>> {
  const MAX_SCHEDULED_POSTS: i64 = 10;
  if let Some(scheduled_publish_time) = scheduled_publish_time {
    let converted = Utc
      .timestamp_opt(scheduled_publish_time, 0)
      .single()
      .ok_or(LemmyErrorType::InvalidUnixTime)?;
    if converted < Utc::now() {
      Err(LemmyErrorType::PostScheduleTimeMustBeInFuture)?;
    }
    if !local_user_view.local_user.admin {
      let count =
        Post::user_scheduled_post_count(local_user_view.person.id, &mut context.pool()).await?;
      if count >= MAX_SCHEDULED_POSTS {
        Err(LemmyErrorType::TooManyScheduledPosts)?;
      }
    }
    Ok(Some(converted))
  } else {
    Ok(None)
  }
}
