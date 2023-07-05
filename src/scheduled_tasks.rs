use chrono::{DateTime, FixedOffset};
use clokwerk::{Scheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::{now, IntervalDsl},
  sql_types::{Integer, Timestamp},
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
  schema::{activity, captcha_answer, comment, community_person_ban, instance, person, post},
  source::instance::{Instance, InstanceForm},
  utils::{naive_now, DELETED_REPLACEMENT_TEXT},
};
use lemmy_routes::nodeinfo::NodeInfo;
use lemmy_utils::{error::LemmyError, REQWEST_TIMEOUT};
use reqwest::blocking::Client;
use std::{thread, time::Duration};
use tracing::{error, info};

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
    let mut conn = PgConnection::establish(&url).expect("could not establish connection");
    active_counts(&mut conn);
    update_banned_when_expired(&mut conn);
  });

  // Update hot ranks every 15 minutes
  let url = db_url.clone();
  scheduler.every(CTimeUnits::minutes(15)).run(move || {
    let mut conn = PgConnection::establish(&url).expect("could not establish connection");
    update_hot_ranks(&mut conn, true);
  });

  // Delete any captcha answers older than ten minutes, every ten minutes
  let url = db_url.clone();
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let mut conn = PgConnection::establish(&url).expect("could not establish connection");
    delete_expired_captcha_answers(&mut conn);
  });

  // Clear old activities every week
  let url = db_url.clone();
  scheduler.every(CTimeUnits::weeks(1)).run(move || {
    let mut conn = PgConnection::establish(&url).expect("could not establish connection");
    clear_old_activities(&mut conn);
  });

  // Remove old rate limit buckets after 1 to 2 hours of inactivity
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let hour = Duration::from_secs(3600);
    context_1.settings_updated_channel().remove_older_than(hour);
  });

  // Overwrite deleted & removed posts and comments every day
  let url = db_url.clone();
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let mut conn = PgConnection::establish(&url).expect("could not establish connection");
    overwrite_deleted_posts_and_comments(&mut conn);
  });

  // Update the Instance Software
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let mut conn = PgConnection::establish(&db_url).expect("could not establish connection");
    update_instance_software(&mut conn, &user_agent);
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
  update_hot_ranks(&mut conn, false);
  update_banned_when_expired(&mut conn);
  clear_old_activities(&mut conn);
  overwrite_deleted_posts_and_comments(&mut conn);
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
fn update_hot_ranks(conn: &mut PgConnection, last_week_only: bool) {
  let process_start_time = if last_week_only {
    info!("Updating hot ranks for last week...");
    naive_now() - chrono::Duration::days(7)
  } else {
    info!("Updating hot ranks for all history...");
    DateTime<FixedOffset>::from_timestamp_opt(0, 0).expect("0 timestamp creation")
  };

  process_hot_ranks_in_batches(
    conn,
    "post_aggregates",
    "SET hot_rank = hot_rank(a.score, a.published),
         hot_rank_active = hot_rank(a.score, a.newest_comment_time_necro)",
    process_start_time,
  );

  process_hot_ranks_in_batches(
    conn,
    "comment_aggregates",
    "SET hot_rank = hot_rank(a.score, a.published)",
    process_start_time,
  );

  process_hot_ranks_in_batches(
    conn,
    "community_aggregates",
    "SET hot_rank = hot_rank(a.subscribers, a.published)",
    process_start_time,
  );

  info!("Finished hot ranks update!");
}

#[derive(QueryableByName)]
struct HotRanksUpdateResult {
  #[diesel(sql_type = Timestamp)]
  published: DateTime<FixedOffset>,
}

/// Runs the hot rank update query in batches until all rows after `process_start_time` have been
/// processed.
/// In `set_clause`, "a" will refer to the current aggregates table.
/// Locked rows are skipped in order to prevent deadlocks (they will likely get updated on the next
/// run)
fn process_hot_ranks_in_batches(
  conn: &mut PgConnection,
  table_name: &str,
  set_clause: &str,
  process_start_time: DateTime<FixedOffset>,
) {
  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    // Raw `sql_query` is used as a performance optimization - Diesel does not support doing this
    // in a single query (neither as a CTE, nor using a subquery)
    let result = sql_query(format!(
      r#"WITH batch AS (SELECT a.id
               FROM {aggregates_table} a
               WHERE a.published > $1
               ORDER BY a.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {aggregates_table} a {set_clause}
             FROM batch WHERE a.id = batch.id RETURNING a.published;
    "#,
      aggregates_table = table_name,
      set_clause = set_clause
    ))
    .bind::<Timestamp, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn);

    match result {
      Ok(updated_rows) => previous_batch_result = updated_rows.last().map(|row| row.published),
      Err(e) => {
        error!("Failed to update {} hot_ranks: {}", table_name, e);
        break;
      }
    }
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {}",
    table_name
  );
}

fn delete_expired_captcha_answers(conn: &mut PgConnection) {
  match diesel::delete(
    captcha_answer::table.filter(captcha_answer::published.lt(now - IntervalDsl::minutes(10))),
  )
  .execute(conn)
  {
    Ok(_) => {
      info!("Done.");
    }
    Err(e) => {
      error!("Failed to clear old captcha answers: {}", e)
    }
  }
}

/// Clear old activities (this table gets very large)
fn clear_old_activities(conn: &mut PgConnection) {
  info!("Clearing old activities...");
  match diesel::delete(activity::table.filter(activity::published.lt(now - 6.months())))
    .execute(conn)
  {
    Ok(_) => {
      info!("Done.");
    }
    Err(e) => {
      error!("Failed to clear old activities: {}", e)
    }
  }
}

/// overwrite posts and comments 30d after deletion
fn overwrite_deleted_posts_and_comments(conn: &mut PgConnection) {
  info!("Overwriting deleted posts...");
  match diesel::update(
    post::table
      .filter(post::deleted.eq(true))
      .filter(post::updated.lt(now.nullable() - 1.months()))
      .filter(post::body.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set((
    post::body.eq(DELETED_REPLACEMENT_TEXT),
    post::name.eq(DELETED_REPLACEMENT_TEXT),
  ))
  .execute(conn)
  {
    Ok(_) => {
      info!("Done.");
    }
    Err(e) => {
      error!("Failed to overwrite deleted posts: {}", e)
    }
  }

  info!("Overwriting deleted comments...");
  match diesel::update(
    comment::table
      .filter(comment::deleted.eq(true))
      .filter(comment::updated.lt(now.nullable() - 1.months()))
      .filter(comment::content.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set(comment::content.eq(DELETED_REPLACEMENT_TEXT))
  .execute(conn)
  {
    Ok(_) => {
      info!("Done.");
    }
    Err(e) => {
      error!("Failed to overwrite deleted comments: {}", e)
    }
  }
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
      "update site_aggregates set users_active_{} = (select * from site_aggregates_activity('{}'))",
      i.1, i.0
    );
    match sql_query(update_site_stmt).execute(conn) {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to update site stats: {}", e)
      }
    }

    let update_community_stmt = format!("update community_aggregates ca set users_active_{} = mv.count_ from community_aggregates_activity('{}') mv where ca.community_id = mv.community_id_", i.1, i.0);
    match sql_query(update_community_stmt).execute(conn) {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to update community stats: {}", e)
      }
    }
  }

  info!("Done.");
}

/// Set banned to false after ban expires
fn update_banned_when_expired(conn: &mut PgConnection) {
  info!("Updating banned column if it expires ...");

  match diesel::update(
    person::table
      .filter(person::banned.eq(true))
      .filter(person::ban_expires.lt(now)),
  )
  .set(person::banned.eq(false))
  .execute(conn)
  {
    Ok(_) => {}
    Err(e) => {
      error!("Failed to update person.banned when expires: {}", e)
    }
  }
  match diesel::delete(community_person_ban::table.filter(community_person_ban::expires.lt(now)))
    .execute(conn)
  {
    Ok(_) => {}
    Err(e) => {
      error!("Failed to remove community_ban expired rows: {}", e)
    }
  }
}

/// Updates the instance software and version
fn update_instance_software(conn: &mut PgConnection, user_agent: &str) {
  info!("Updating instances software and versions...");

  let client = match Client::builder()
    .user_agent(user_agent)
    .timeout(REQWEST_TIMEOUT)
    .build()
  {
    Ok(client) => client,
    Err(e) => {
      error!("Failed to build reqwest client: {}", e);
      return;
    }
  };

  let instances = match instance::table.get_results::<Instance>(conn) {
    Ok(instances) => instances,
    Err(e) => {
      error!("Failed to get instances: {}", e);
      return;
    }
  };

  for instance in instances {
    let node_info_url = format!("https://{}/nodeinfo/2.0.json", instance.domain);

    // Skip it if it can't connect
    let res = client
      .get(&node_info_url)
      .send()
      .ok()
      .and_then(|t| t.json::<NodeInfo>().ok());

    if let Some(node_info) = res {
      let software = node_info.software.as_ref();
      let form = InstanceForm::builder()
        .domain(instance.domain)
        .software(software.and_then(|s| s.name.clone()))
        .version(software.and_then(|s| s.version.clone()))
        .updated(Some(naive_now()))
        .build();

      match diesel::update(instance::table.find(instance.id))
        .set(form)
        .execute(conn)
      {
        Ok(_) => {
          info!("Done.");
        }
        Err(e) => {
          error!("Failed to update site instance software: {}", e);
          return;
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
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
