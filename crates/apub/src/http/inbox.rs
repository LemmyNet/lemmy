use crate::{
  activities::WithPublished,
  activity_lists::SharedInboxActivities,
  fetcher::user_or_community::UserOrCommunity,
};
use activitypub_federation::{
  actix_web::inbox::{receive_activity, receive_activity_parts},
  config::Data,
};
use actix_web::{http::header::HeaderMap, web::Bytes, HttpRequest, HttpResponse};
use chrono::{DateTime, Local, TimeDelta, Utc};
use http::{Method, Uri};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{newtypes::InstanceId, source::instance::Instance};
use lemmy_utils::error::LemmyResult;
use once_cell::sync::Lazy;
use rand::seq::IteratorRandom;
use serde::Deserialize;
use std::{
  cmp::Ordering,
  collections::{BinaryHeap, HashMap},
  sync::{Arc, RwLock},
  thread::available_parallelism,
  time::Duration,
};
use tokio::{spawn, task::JoinHandle, time::sleep};
use tracing::info;
use url::Url;

/// Handle incoming activities.
pub async fn shared_inbox(
  request: HttpRequest,
  bytes: Bytes,
  data: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  match serde_json::from_slice::<WithPublished<()>>(&bytes)?.published {
    Some(published) => {
      // includes published timestamp, insert to queue to ensure that activities are processed
      // in correct order even when delivered out of order.
      let request_parts = (
        request.headers().clone(),
        request.method().clone(),
        request.uri().clone(),
      );

      #[derive(Deserialize)]
      struct Id {
        id: Url,
      }

      let activity_id = serde_json::from_slice::<Id>(&bytes)?.id;
      let domain = activity_id.domain().unwrap().to_string();
      let instance = Instance::read_or_create(&mut data.pool(), domain)
        .await
        .unwrap();

      let mut lock = ACTIVITY_QUEUE.write().unwrap();
      let instance_queue = lock.entry(instance.id).or_insert(BinaryHeap::new());
      while instance_queue.len() > 5 {
        // TODO: must not hold lock here
        sleep(Duration::from_millis(100)).await;
      }
      instance_queue.push(InboxActivity {
        request_parts,
        bytes,
        published,
      });
      Ok(HttpResponse::Ok().finish())
    }
    None => {
      // no timestamp included, process immediately
      receive_activity::<SharedInboxActivities, UserOrCommunity, LemmyContext>(
        request, bytes, &data,
      )
      .await
    }
  }
}

/// Queue of incoming activities, ordered by oldest published first
static ACTIVITY_QUEUE: Lazy<Arc<RwLock<HashMap<InstanceId, BinaryHeap<InboxActivity>>>>> =
  Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Minimum age of an activity before it gets processed. This ensures that an activity which was
/// delayed still gets processed in correct order.
const RECEIVE_DELAY: Option<TimeDelta> = TimeDelta::try_seconds(1);

pub fn handle_received_activities(
  context: &Data<LemmyContext>,
) -> LemmyResult<Vec<JoinHandle<()>>> {
  // launch one task per cpu core
  let parallelism = available_parallelism()?.into();
  let workers = (0..parallelism)
    .map(|_| {
      let context = context.reset_request_count();
      spawn(async move {
        loop {
          let now = Local::now();
          let instance_id = {
            let lock = ACTIVITY_QUEUE.read().unwrap();
            lock.keys().choose(&mut rand::thread_rng()).unwrap().clone()
          };
          if let Some(latest_timestamp) = peek_queue_timestamp(&instance_id) {
            if latest_timestamp < now - RECEIVE_DELAY.unwrap() {
              if let Some(a) = pop_queue(&instance_id) {
                let parts = (&a.request_parts.0, &a.request_parts.1, &a.request_parts.2);
                receive_activity_parts::<SharedInboxActivities, UserOrCommunity, LemmyContext>(
                  parts, a.bytes, &context,
                )
                .await
                .inspect_err(|e| info!("Error receiving activity: {e}"))
                .ok();
              }
            }
          }
          // TODO: could sleep based on remaining time until head activity reaches 1s
          //       or simply use `WORK_FINISHED_RECHECK_DELAY` from lemmy_federate
          sleep(Duration::from_millis(100)).await;
          // TODO: need cancel? lemmy seems to shutdown just fine
        }
      })
    })
    .collect();

  Ok(workers)
}

fn peek_queue_timestamp(instance_id: &InstanceId) -> Option<DateTime<Utc>> {
  ACTIVITY_QUEUE
    .read()
    .unwrap()
    .get(instance_id)
    .unwrap()
    .peek()
    .map(|i| i.published)
}

fn pop_queue<'a>(instance_id: &InstanceId) -> Option<InboxActivity> {
  let mut lock = ACTIVITY_QUEUE.write().unwrap();
  let res = lock.get_mut(instance_id).unwrap().pop();
  if lock.is_empty() {
    lock.remove(instance_id);
  }
  res
}

#[derive(Clone, Debug)]
struct InboxActivity {
  // Need to store like this because HttpRequest is not Sync
  request_parts: (HeaderMap, Method, Uri),
  bytes: Bytes,
  published: DateTime<Utc>,
}

impl PartialOrd for InboxActivity {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    other.published.partial_cmp(&self.published)
  }
}

impl Ord for InboxActivity {
  fn cmp(&self, other: &Self) -> Ordering {
    other.published.cmp(&self.published)
  }
}

impl PartialEq for InboxActivity {
  fn eq(&self, other: &Self) -> bool {
    self.bytes.eq(&other.bytes)
  }
}

impl Eq for InboxActivity {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn activity_queue_order() {
    let activity1 = InboxActivity {
      request_parts: Default::default(),
      bytes: Default::default(),
      published: Local::now().into(),
    };
    let activity2 = InboxActivity {
      request_parts: Default::default(),
      bytes: Default::default(),
      published: Local::now().into(),
    };
    let activity3 = InboxActivity {
      request_parts: Default::default(),
      bytes: Default::default(),
      published: Local::now().into(),
    };
    let mut lock = ACTIVITY_QUEUE.write().unwrap();

    // insert in wrong order
    lock.push(activity3.clone());
    lock.push(activity1.clone());
    lock.push(activity2.clone());

    // should be popped in correct order
    assert_eq!(activity1.published, lock.pop().unwrap().published);
    assert_eq!(activity2.published, lock.pop().unwrap().published);
    assert_eq!(activity3.published, lock.pop().unwrap().published);
  }
}
