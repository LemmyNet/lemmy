use crate::{
  federation_queue_state::FederationQueueState,
  util::{
    get_activity_cached,
    get_actor_cached,
    get_latest_activity_id,
    intern_url,
    retry_sleep_duration,
  },
};
use activitypub_federation::{
  activity_queue::{prepare_raw, send_raw, sign_raw},
  config::Data,
};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use lemmy_db_schema::{
  newtypes::{CommunityId, InstanceId},
  source::{activity::SentActivity, instance::Instance, site::Site},
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::{error::LemmyErrorExt2, REQWEST_TIMEOUT};
use reqwest::Url;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  ops::Deref,
  sync::Arc,
  time::Duration,
};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tokio_util::sync::CancellationToken;
/// save state to db every n sends if there's no failures (otherwise state is saved after every attempt)
static SAVE_STATE_EVERY_IT: i64 = 100;
static SAVE_STATE_EVERY_TIME: Duration = Duration::from_secs(10);

/// loop fetch new activities from db and send them to the inboxes of the given instances
/// this worker only returns if (a) there is an internal error or (b) the cancellation token is cancelled (graceful exit)
pub async fn instance_worker(
  pool: ActualDbPool,
  instance: Instance,
  data: Data<()>,
  stop: CancellationToken,
  stats_sender: UnboundedSender<FederationQueueState>,
) -> Result<(), anyhow::Error> {
  let mut pool = &mut DbPool::Pool(&pool);
  let mut last_full_communities_fetch = Utc.timestamp_nanos(0);
  let mut last_incremental_communities_fetch = Utc.timestamp_nanos(0);
  let mut last_state_insert = Utc.timestamp_nanos(0);
  let mut followed_communities: HashMap<CommunityId, HashSet<Arc<Url>>> = get_communities(
    &mut pool,
    instance.id,
    &mut last_incremental_communities_fetch,
  )
  .await?;
  let site = Site::read_from_instance_id(&mut pool, instance.id).await?;

  let mut state = FederationQueueState::load(&mut pool, &instance.domain).await?;
  if state.fail_count > 0 {
    // before starting queue, sleep remaining duration
    let elapsed = (Utc::now() - state.last_retry).to_std()?;
    let remaining = retry_sleep_duration(state.fail_count) - elapsed;
    tokio::select! {
      () = sleep(remaining) => {},
      () = stop.cancelled() => { return Ok(()); }
    }
  }
  while !stop.is_cancelled() {
    let latest_id = get_latest_activity_id(&mut pool).await?;
    let mut id = state.last_successful_id;
    if id == latest_id {
      // no more work to be done, wait before rechecking
      tokio::select! {
        () = sleep(Duration::from_secs(10)) => { continue; },
        () = stop.cancelled() => { return Ok(()); }
      }
    }
    let mut processed_activities = 0;
    'batch: while id < latest_id
      && processed_activities < SAVE_STATE_EVERY_IT
      && !stop.is_cancelled()
    {
      id += 1;
      processed_activities += 1;
      let Some(ele) = get_activity_cached(&mut pool, id).await? else {
        state.last_successful_id = id;
        continue;
      };
      let (activity, object) = (&ele.0, &ele.1);
      let inbox_urls = get_inbox_urls(&instance, &site, &followed_communities, activity);
      if inbox_urls.is_empty() {
        state.last_successful_id = id;
        continue;
      }
      let actor = get_actor_cached(
        &mut pool,
        activity.actor_type,
        activity.actor_apub_id.deref(),
      )
      .await?;

      let inbox_urls = inbox_urls.into_iter().map(|e| (*e).clone()).collect();
      let requests = prepare_raw(object, actor.as_ref(), inbox_urls, &data)
        .await
        .into_anyhow()?;
      for task in requests {
        // usually only one due to shared inbox
        let mut req = sign_raw(&task, &data, REQWEST_TIMEOUT).await?;
        tracing::info!("sending out {}", task);
        while let Err(e) = send_raw(&task, &data, req).await {
          tracing::info!("{task} failed: {e}");
          state.fail_count += 1;
          state.last_retry = Utc::now();
          stats_sender.send(state.clone())?;
          FederationQueueState::upsert(&mut pool, &state).await?;
          req = sign_raw(&task, &data, REQWEST_TIMEOUT).await?; // resign request
          tokio::select! {
            () = sleep(retry_sleep_duration(state.fail_count)) => {},
            () = stop.cancelled() => {
              // save state to db and exit
              break 'batch;
            }
          }
        }
      }
      // send success!
      state.last_successful_id = id;
      state.fail_count = 0;
    }

    if Utc::now() - last_state_insert > chrono::Duration::from_std(SAVE_STATE_EVERY_TIME).unwrap() {
      last_state_insert = Utc::now();
      FederationQueueState::upsert(&mut pool, &state).await?;
      stats_sender.send(state.clone())?;
    }
    {
      // update communities
      if (Utc::now() - last_incremental_communities_fetch) > chrono::Duration::seconds(10) {
        // process additions every 10s
        followed_communities.extend(
          get_communities(
            &mut pool,
            instance.id,
            &mut last_incremental_communities_fetch,
          )
          .await?,
        );
      }
      if (Utc::now() - last_full_communities_fetch) > chrono::Duration::seconds(300) {
        // process removals every 5min
        last_full_communities_fetch = Utc.timestamp_nanos(0);
        followed_communities =
          get_communities(&mut pool, instance.id, &mut last_full_communities_fetch).await?;
        last_incremental_communities_fetch = last_full_communities_fetch.clone();
      }
    }
  }

  Ok(())
}

/// get inbox urls of sending the given activity to the given instance
/// most often this will return 0 values (if instance doesn't care about the activity)
/// or 1 value (the shared inbox)
/// > 1 values only happens for non-lemmy software
fn get_inbox_urls(
  instance: &Instance,
  site: &Option<Site>,
  followed_communities: &HashMap<CommunityId, HashSet<Arc<Url>>>,
  activity: &SentActivity,
) -> HashSet<Arc<Url>> {
  let mut inbox_urls = HashSet::new();
  let targets = &activity.send_targets;
  if targets.all_instances {
    if let Some(site) = &site {
      // todo: when does an instance not have a site?
      inbox_urls.insert(intern_url(Cow::Borrowed(site.inbox_url.deref())));
    }
  }
  for t in &targets.community_followers_of {
    if let Some(urls) = followed_communities.get(t) {
      inbox_urls.extend(urls.iter().map(|e| e.clone()));
    }
  }
  for inbox in &targets.inboxes {
    if inbox.domain() != Some(&instance.domain) {
      continue;
    }
    inbox_urls.insert(intern_url(Cow::Borrowed(inbox)));
  }
  inbox_urls
}

/// get a list of local communities with the remote inboxes on the given instance that cares about them
async fn get_communities(
  pool: &mut DbPool<'_>,
  instance_id: InstanceId,
  last_fetch: &mut DateTime<Utc>,
) -> Result<HashMap<CommunityId, HashSet<Arc<Url>>>> {
  let e = *last_fetch;
  *last_fetch = Utc::now(); // update to time before fetch to ensure overlap
  Ok(
    CommunityFollowerView::get_instance_followed_community_inboxes(pool, instance_id, e)
      .await?
      .into_iter()
      .fold(HashMap::new(), |mut map, (c, u)| {
        map
          .entry(c)
          .or_insert_with(|| HashSet::new())
          .insert(intern_url(Cow::Owned(u.into())));
        map
      }),
  )
}
