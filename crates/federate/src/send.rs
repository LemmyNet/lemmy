use crate::util::get_actor_cached;
use activitypub_federation::{
  activity_sending::SendActivityTask,
  config::Data,
  protocol::context::WithContext,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, federate_retry_sleep_duration};
use lemmy_apub::{activity_lists::SharedInboxActivities, FEDERATION_CONTEXT};
use lemmy_db_schema::{newtypes::ActivityId, source::activity::SentActivity};
use reqwest::Url;
use std::ops::Deref;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Eq)]
pub(crate) struct SendSuccessInfo {
  pub activity_id: ActivityId,
  pub published: Option<DateTime<Utc>>,
  pub was_skipped: bool,
}
/// order backwards by activity_id for the binary heap in the worker
impl PartialEq for SendSuccessInfo {
  fn eq(&self, other: &Self) -> bool {
    self.activity_id == other.activity_id
  }
}
/// order backwards because the binary heap is a max heap, and we need the smallest element to be on
/// top
impl PartialOrd for SendSuccessInfo {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for SendSuccessInfo {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    other.activity_id.cmp(&self.activity_id)
  }
}

/// Represents the result of sending an activity.
///
/// This enum is used to communicate the outcome of a send operation from a send task
/// to the main instance worker. It's designed to maintain a clean separation between
/// the send task and the main thread, allowing the send.rs file to be self-contained
/// and easier to understand.
///
/// The use of a channel for communication (rather than shared atomic variables) was chosen
/// because:
/// 1. It keeps the send task cleanly separated with no direct interaction with the main thread.
/// 2. The failure event needs to be transferred to the main task for database updates anyway.
/// 3. The main fail_count should only be updated under certain conditions, which are best handled
///    in the main task.
/// 4. It maintains consistency in how data is communicated (all via channels rather than a mix of
///    channels and atomics).
/// 5. It simplifies concurrency management and makes the flow of data more predictable.
pub(crate) enum SendActivityResult {
  Success(SendSuccessInfo),
  Failure { fail_count: i32 },
}
/// Represents a task for retrying to send an activity.
///
/// This struct encapsulates all the necessary information and resources for attempting
/// to send an activity to multiple inbox URLs, with built-in retry logic.
pub(crate) struct SendRetryTask<'a> {
  pub activity: &'a SentActivity,
  pub object: &'a SharedInboxActivities,
  /// Must not be empty at this point
  pub inbox_urls: Vec<Url>,
  /// Channel to report results back to the main instance worker
  pub report: &'a mut UnboundedSender<SendActivityResult>,
  /// The first request will be sent immediately, but subsequent requests will be delayed
  /// according to the number of previous fails + 1
  ///
  /// This is a read-only immutable variable that is passed only one way, from the main
  /// thread to each send task. It allows the task to determine how long to sleep initially
  /// if the request fails.
  pub initial_fail_count: i32,
  /// For logging purposes
  pub domain: String,
  pub context: Data<LemmyContext>,
  pub stop: CancellationToken,
}

impl<'a> SendRetryTask<'a> {
  // this function will return successfully when (a) send succeeded or (b) worker cancelled
  // and will return an error if an internal error occurred (send errors cause an infinite loop)
  pub async fn send_retry_loop(self) -> Result<()> {
    let SendRetryTask {
      activity,
      object,
      inbox_urls,
      report,
      initial_fail_count,
      domain,
      context,
      stop,
    } = self;
    debug_assert!(!inbox_urls.is_empty());

    let pool = &mut context.pool();
    let Some(actor_apub_id) = &activity.actor_apub_id else {
      return Err(anyhow::anyhow!("activity is from before lemmy 0.19"));
    };
    let actor = get_actor_cached(pool, activity.actor_type, actor_apub_id)
      .await
      .context("failed getting actor instance (was it marked deleted / removed?)")?;

    let object = WithContext::new(object.clone(), FEDERATION_CONTEXT.deref().clone());
    let requests = SendActivityTask::prepare(&object, actor.as_ref(), inbox_urls, &context).await?;
    for task in requests {
      // usually only one due to shared inbox
      tracing::debug!("sending out {}", task);
      let mut fail_count = initial_fail_count;
      while let Err(e) = task.sign_and_send(&context).await {
        fail_count += 1;
        report.send(SendActivityResult::Failure {
          fail_count,
          // activity_id: activity.id,
        })?;
        let retry_delay = federate_retry_sleep_duration(fail_count);
        tracing::info!(
          "{}: retrying {:?} attempt {} with delay {retry_delay:.2?}. ({e})",
          domain,
          activity.id,
          fail_count
        );
        tokio::select! {
          () = sleep(retry_delay) => {},
          () = stop.cancelled() => {
            // cancel sending without reporting any result.
            // the InstanceWorker needs to be careful to not hang on receive of that
            // channel when cancelled (see handle_send_results)
            return Ok(());
          }
        }
      }
    }
    report.send(SendActivityResult::Success(SendSuccessInfo {
      activity_id: activity.id,
      published: Some(activity.published),
      was_skipped: false,
    }))?;
    Ok(())
  }
}
