use chrono::{DateTime, TimeZone, Utc};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

pub mod create;
pub mod delete;
pub mod read;
pub mod remove;
pub mod update;

fn convert_published_time(
  scheduled_publish_time: Option<i64>,
) -> LemmyResult<Option<DateTime<Utc>>> {
  if let Some(scheduled_publish_time) = scheduled_publish_time {
    let converted = Utc
      .timestamp_opt(scheduled_publish_time, 0)
      .single()
      .ok_or(LemmyErrorType::InvalidUnixTime)?;
    if converted < Utc::now() {
      Err(LemmyErrorType::PostScheduleTimeMustBeInFuture)?;
    }
    Ok(Some(converted))
  } else {
    Ok(None)
  }
}
