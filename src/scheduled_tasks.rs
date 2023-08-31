use chrono::{DateTime, TimeZone, Utc};
use clokwerk::{Scheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::IntervalDsl,
  sql_types::{Integer, Timestamptz},
  Connection,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  QueryableByName,
};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  schema::{
    captcha_answer,
    comment,
    community_person_ban,
    instance,
    person,
    post,
    received_activity,
    sent_activity,
  },
  source::instance::{Instance, InstanceForm},
  utils::{naive_now, now, DELETED_REPLACEMENT_TEXT},
};
use lemmy_routes::nodeinfo::NodeInfo;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  REQWEST_TIMEOUT,
};
use reqwest::blocking::Client;
use std::{thread, time::Duration};
use tracing::{error, info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub fn setup(
  db_url: String,
  user_agent: String,
  context_1: LemmyContext,
) -> Result<(), LemmyError> {
  // Setup the connections
  let mut scheduler = Scheduler::new();

  startup_jobs(&db_url);

  // Update active counts every hour
  let url = db_url.clone();
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    PgConnection::establish(&url)
      .map(|mut conn| {
        active_counts(&mut conn);
        update_banned_when_expired(&mut conn);
      })
      .map_err(|e| {
        error!("Failed to establish db connection for active counts update: {e}");
      })
      .ok();
  });

  // Update hot ranks every 15 minutes
  let url = db_url.clone();
  scheduler.every(CTimeUnits::minutes(15)).run(move || {
    PgConnection::establish(&url)
      .map(|mut conn| {
        update_hot_ranks(&mut conn);
      })
      .map_err(|e| {
        error!("Failed to establish db connection for hot ranks update: {e}");
      })
      .ok();
  });

  // Delete any captcha answers older than ten minutes, every ten minutes
  let url = db_url.clone();
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    PgConnection::establish(&url)
      .map(|mut conn| {
        delete_expired_captcha_answers(&mut conn);
      })
      .map_err(|e| {
        error!("Failed to establish db connection for captcha cleanup: {e}");
      })
      .ok();
  });

  // Clear old activities every week
  let url = db_url.clone();
  scheduler.every(CTimeUnits::weeks(1)).run(move || {
    PgConnection::establish(&url)
      .map(|mut conn| {
        clear_old_activities(&mut conn);
      })
      .map_err(|e| {
        error!("Failed to establish db connection for activity cleanup: {e}");
      })
      .ok();
  });

  // Remove old rate limit buckets after 1 to 2 hours of inactivity
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let hour = Duration::from_secs(3600);
    context_1.settings_updated_channel().remove_older_than(hour);
  });

  // Overwrite deleted & removed posts and comments every day
  let url = db_url.clone();
  scheduler.every(CTimeUnits::days(1)).run(move || {
    PgConnection::establish(&db_url)
      .map(|mut conn| {
        overwrite_deleted_posts_and_comments(&mut conn);
      })
      .map_err(|e| {
        error!("Failed to establish db connection for deleted content cleanup: {e}");
      })
      .ok();
  });

  // Update the Instance Software
  scheduler.every(CTimeUnits::days(1)).run(move || {
    PgConnection::establish(&url)
      .map(|mut conn| {
        update_instance_software(&mut conn, &user_agent)
          .map_err(|e| warn!("Failed to update instance software: {e}"))
          .ok();
      })
      .map_err(|e| {
        error!("Failed to establish db connection for instance software update: {e}");
      })
      .ok();
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending();
    thread::sleep(Duration::from_millis(1000));
  }
}

/// Run these on server startup
fn startup_jobs(db_url: &str) {
  let mut conn = PgConnection::establish(db_url).expect("could not establish connection");
  active_counts(&mut conn);
  update_hot_ranks(&mut conn);
  update_banned_when_expired(&mut conn);
  clear_old_activities(&mut conn);
  overwrite_deleted_posts_and_comments(&mut conn);
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
fn update_hot_ranks(conn: &mut PgConnection) {
  info!("Updating hot ranks for all history...");

  process_hot_ranks_in_batches(
    conn,
    "post_aggregates",
    "a.hot_rank != 0 OR a.hot_rank_active != 0",
    "SET hot_rank = hot_rank(a.score, a.published),
         hot_rank_active = hot_rank(a.score, a.newest_comment_time_necro)",
  );

  process_hot_ranks_in_batches(
    conn,
    "comment_aggregates",
    "a.hot_rank != 0",
    "SET hot_rank = hot_rank(a.score, a.published)",
  );

  process_hot_ranks_in_batches(
    conn,
    "community_aggregates",
    "a.hot_rank != 0",
    "SET hot_rank = hot_rank(a.subscribers, a.published)",
  );

  info!("Finished hot ranks update!");
}

#[derive(QueryableByName)]
struct HotRanksUpdateResult {
  #[diesel(sql_type = Timestamptz)]
  published: DateTime<Utc>,
}

/// Runs the hot rank update query in batches until all rows have been processed.
/// In `where_clause` and `set_clause`, "a" will refer to the current aggregates table.
/// Locked rows are skipped in order to prevent deadlocks (they will likely get updated on the next
/// run)
fn process_hot_ranks_in_batches(
  conn: &mut PgConnection,
  table_name: &str,
  where_clause: &str,
  set_clause: &str,
) {
  let process_start_time: DateTime<Utc> = Utc
    .timestamp_opt(0, 0)
    .single()
    .expect("0 timestamp creation");

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    // Raw `sql_query` is used as a performance optimization - Diesel does not support doing this
    // in a single query (neither as a CTE, nor using a subquery)
    let result = sql_query(format!(
      r#"WITH batch AS (SELECT a.id
               FROM {aggregates_table} a
               WHERE a.published > $1 AND ({where_clause})
               ORDER BY a.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {aggregates_table} a {set_clause}
             FROM batch WHERE a.id = batch.id RETURNING a.published;
    "#,
      aggregates_table = table_name,
      set_clause = set_clause,
      where_clause = where_clause
    ))
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn);

    match result {
      Ok(updated_rows) => {
        processed_rows_count += updated_rows.len();
        previous_batch_result = updated_rows.last().map(|row| row.published);
      }
      Err(e) => {
        error!("Failed to update {} hot_ranks: {}", table_name, e);
        break;
      }
    }
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    table_name, processed_rows_count
  );
}

fn delete_expired_captcha_answers(conn: &mut PgConnection) {
  diesel::delete(
    captcha_answer::table.filter(captcha_answer::published.lt(now() - IntervalDsl::minutes(10))),
  )
  .execute(conn)
  .map(|_| {
    info!("Done.");
  })
  .map_err(|e| error!("Failed to clear old captcha answers: {e}"))
  .ok();
}

/// Clear old activities (this table gets very large)
fn clear_old_activities(conn: &mut PgConnection) {
  info!("Clearing old activities...");
  diesel::delete(sent_activity::table.filter(sent_activity::published.lt(now() - 3.months())))
    .execute(conn)
    .map_err(|e| error!("Failed to clear old sent activities: {e}"))
    .ok();

  diesel::delete(
    received_activity::table.filter(received_activity::published.lt(now() - 3.months())),
  )
  .execute(conn)
  .map(|_| info!("Done."))
  .map_err(|e| error!("Failed to clear old received activities: {e}"))
  .ok();
}

/// overwrite posts and comments 30d after deletion
fn overwrite_deleted_posts_and_comments(conn: &mut PgConnection) {
  info!("Overwriting deleted posts...");
  diesel::update(
    post::table
      .filter(post::deleted.eq(true))
      .filter(post::updated.lt(now().nullable() - 1.months()))
      .filter(post::body.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set((
    post::body.eq(DELETED_REPLACEMENT_TEXT),
    post::name.eq(DELETED_REPLACEMENT_TEXT),
  ))
  .execute(conn)
  .map(|_| {
    info!("Done.");
  })
  .map_err(|e| error!("Failed to overwrite deleted posts: {e}"))
  .ok();

  info!("Overwriting deleted comments...");
  diesel::update(
    comment::table
      .filter(comment::deleted.eq(true))
      .filter(comment::updated.lt(now().nullable() - 1.months()))
      .filter(comment::content.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set(comment::content.eq(DELETED_REPLACEMENT_TEXT))
  .execute(conn)
  .map(|_| {
    info!("Done.");
  })
  .map_err(|e| error!("Failed to overwrite deleted comments: {e}"))
  .ok();
}

/// Re-calculate the site and community active counts every 12 hours
fn active_counts(conn: &mut PgConnection) {
  info!("Updating active site and community aggregates ...");

  let intervals = vec![
    ("1 day", "day"),
    ("1 week", "week"),
    ("1 month", "month"),
    ("6 months", "half_year"),
  ];

  for i in &intervals {
    let update_site_stmt = format!(
      "update site_aggregates set users_active_{} = (select * from site_aggregates_activity('{}')) where site_id = 1",
      i.1, i.0
    );
    sql_query(update_site_stmt)
      .execute(conn)
      .map_err(|e| error!("Failed to update site stats: {e}"))
      .ok();

    let update_community_stmt = format!("update community_aggregates ca set users_active_{} = mv.count_ from community_aggregates_activity('{}') mv where ca.community_id = mv.community_id_", i.1, i.0);
    sql_query(update_community_stmt)
      .execute(conn)
      .map_err(|e| error!("Failed to update community stats: {e}"))
      .ok();
  }

  info!("Done.");
}

/// Set banned to false after ban expires
fn update_banned_when_expired(conn: &mut PgConnection) {
  info!("Updating banned column if it expires ...");

  diesel::update(
    person::table
      .filter(person::banned.eq(true))
      .filter(person::ban_expires.lt(now().nullable())),
  )
  .set(person::banned.eq(false))
  .execute(conn)
  .map_err(|e| error!("Failed to update person.banned when expires: {e}"))
  .ok();

  diesel::delete(
    community_person_ban::table.filter(community_person_ban::expires.lt(now().nullable())),
  )
  .execute(conn)
  .map_err(|e| error!("Failed to remove community_ban expired rows: {e}"))
  .ok();
}

/// Updates the instance software and version
///
/// TODO: this should be async
/// TODO: if instance has been dead for a long time, it should be checked less frequently
fn update_instance_software(conn: &mut PgConnection, user_agent: &str) -> LemmyResult<()> {
  info!("Updating instances software and versions...");

  let client = Client::builder()
    .user_agent(user_agent)
    .timeout(REQWEST_TIMEOUT)
    .connect_timeout(REQWEST_TIMEOUT)
    .build()?;

  let instances = instance::table.get_results::<Instance>(conn)?;

  for instance in instances {
    let node_info_url = format!("https://{}/nodeinfo/2.0.json", instance.domain);

    // The `updated` column is used to check if instances are alive. If it is more than three days
    // in the past, no outgoing activities will be sent to that instance. However not every
    // Fediverse instance has a valid Nodeinfo endpoint (its not required for Activitypub). That's
    // why we always need to mark instances as updated if they are alive.
    let default_form = InstanceForm::builder()
      .domain(instance.domain.clone())
      .updated(Some(naive_now()))
      .build();
    let form = match client.get(&node_info_url).send() {
      Ok(res) if res.status().is_client_error() => {
        // Instance doesnt have nodeinfo but sent a response, consider it alive
        Some(default_form)
      }
      Ok(res) => match res.json::<NodeInfo>() {
        Ok(node_info) => {
          // Instance sent valid nodeinfo, write it to db
          let software = node_info.software.as_ref();
          Some(
            InstanceForm::builder()
              .domain(instance.domain)
              .updated(Some(naive_now()))
              .software(software.and_then(|s| s.name.clone()))
              .version(software.and_then(|s| s.version.clone()))
              .build(),
          )
        }
        Err(_) => {
          // No valid nodeinfo but valid HTTP response, consider instance alive
          Some(default_form)
        }
      },
      Err(_) => {
        // dead instance, do nothing
        None
      }
    };
    if let Some(form) = form {
      diesel::update(instance::table.find(instance.id))
        .set(form)
        .execute(conn)?;
    }
  }
  info!("Finished updating instances software and versions...");
  Ok(())
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use lemmy_routes::nodeinfo::NodeInfo;
  use reqwest::Client;

  #[tokio::test]
  #[ignore]
  async fn test_nodeinfo() {
    let client = Client::builder().build().unwrap();
    let lemmy_ml_nodeinfo = client
      .get("https://lemmy.ml/nodeinfo/2.0.json")
      .send()
      .await
      .unwrap()
      .json::<NodeInfo>()
      .await
      .unwrap();

    assert_eq!(lemmy_ml_nodeinfo.software.unwrap().name.unwrap(), "lemmy");
  }
}
