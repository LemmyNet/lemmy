use chrono::{DateTime, TimeZone, Utc};
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::IntervalDsl,
  sql_query,
  sql_types::{Integer, Timestamptz},
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  QueryableByName,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
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
  utils::{get_conn, naive_now, now, DbPool, DELETED_REPLACEMENT_TEXT},
};
use lemmy_routes::nodeinfo::NodeInfo;
use lemmy_utils::error::{LemmyError, LemmyResult};
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;
use tracing::{error, info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: LemmyContext) -> Result<(), LemmyError> {
  // Setup the connections
  let mut scheduler = AsyncScheduler::new();
  startup_jobs(&mut context.pool()).await;

  let context_1 = context.clone();
  // Update active counts every hour
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let context = context_1.clone();

    async move {
      active_counts(&mut context.pool()).await;
      update_banned_when_expired(&mut context.pool()).await;
    }
  });

  let context_1 = context.clone();
  // Update hot ranks every 15 minutes
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.clone();

    async move {
      update_hot_ranks(&mut context.pool()).await;
    }
  });

  let context_1 = context.clone();
  // Delete any captcha answers older than ten minutes, every ten minutes
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.clone();

    async move {
      delete_expired_captcha_answers(&mut context.pool()).await;
    }
  });

  let context_1 = context.clone();
  // Clear old activities every week
  scheduler.every(CTimeUnits::weeks(1)).run(move || {
    let context = context_1.clone();

    async move {
      clear_old_activities(&mut context.pool()).await;
    }
  });

  let context_1 = context.clone();
  // Remove old rate limit buckets after 1 to 2 hours of inactivity
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let context = context_1.clone();

    async move {
      let hour = Duration::from_secs(3600);
      context.settings_updated_channel().remove_older_than(hour);
    }
  });

  let context_1 = context.clone();
  // Overwrite deleted & removed posts and comments every day
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let context = context_1.clone();

    async move {
      overwrite_deleted_posts_and_comments(&mut context.pool()).await;
    }
  });

  let context_1 = context.clone();
  // Update the Instance Software
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let context = context_1.clone();

    async move {
      update_instance_software(&mut context.pool(), context.client())
        .await
        .map_err(|e| warn!("Failed to update instance software: {e}"))
        .ok();
    }
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

/// Run these on server startup
async fn startup_jobs(pool: &mut DbPool<'_>) {
  active_counts(pool).await;
  update_hot_ranks(pool).await;
  update_banned_when_expired(pool).await;
  clear_old_activities(pool).await;
  overwrite_deleted_posts_and_comments(pool).await;
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
async fn update_hot_ranks(pool: &mut DbPool<'_>) {
  info!("Updating hot ranks for all history...");

  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
      process_post_aggregates_ranks_in_batches(&mut conn).await;

      process_ranks_in_batches(
        &mut conn,
        "comment_aggregates",
        "a.hot_rank != 0",
        "SET hot_rank = hot_rank(a.score, a.published)",
      )
      .await;

      process_ranks_in_batches(
        &mut conn,
        "community_aggregates",
        "a.hot_rank != 0",
        "SET hot_rank = hot_rank(a.subscribers, a.published)",
      )
      .await;

      info!("Finished hot ranks update!");
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
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
async fn process_ranks_in_batches(
  conn: &mut AsyncPgConnection,
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
    .get_results::<HotRanksUpdateResult>(conn)
    .await;

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

/// Post aggregates is a special case, since it needs to join to the community_aggregates
/// table, to get the active monthly user counts.
async fn process_post_aggregates_ranks_in_batches(conn: &mut AsyncPgConnection) {
  let process_start_time: DateTime<Utc> = Utc
    .timestamp_opt(0, 0)
    .single()
    .expect("0 timestamp creation");

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    let result = sql_query(
      r"WITH batch AS (SELECT pa.id
               FROM post_aggregates pa
               WHERE pa.published > $1
               AND (pa.hot_rank != 0 OR pa.hot_rank_active != 0)
               ORDER BY pa.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE post_aggregates pa
           SET hot_rank = hot_rank(pa.score, pa.published),
           hot_rank_active = hot_rank(pa.score, pa.newest_comment_time_necro),
           scaled_rank = scaled_rank(pa.score, pa.published, ca.users_active_month)
         FROM batch, community_aggregates ca
         WHERE pa.id = batch.id and pa.community_id = ca.community_id RETURNING pa.published;
    ",
    )
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn)
    .await;

    match result {
      Ok(updated_rows) => {
        processed_rows_count += updated_rows.len();
        previous_batch_result = updated_rows.last().map(|row| row.published);
      }
      Err(e) => {
        error!("Failed to update {} hot_ranks: {}", "post_aggregates", e);
        break;
      }
    }
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    "post_aggregates", processed_rows_count
  );
}

async fn delete_expired_captcha_answers(pool: &mut DbPool<'_>) {
  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
      diesel::delete(
        captcha_answer::table
          .filter(captcha_answer::published.lt(now() - IntervalDsl::minutes(10))),
      )
      .execute(&mut conn)
      .await
      .map(|_| {
        info!("Done.");
      })
      .map_err(|e| error!("Failed to clear old captcha answers: {e}"))
      .ok();
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
}

/// Clear old activities (this table gets very large)
async fn clear_old_activities(pool: &mut DbPool<'_>) {
  info!("Clearing old activities...");
  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
      diesel::delete(sent_activity::table.filter(sent_activity::published.lt(now() - 3.months())))
        .execute(&mut conn)
        .await
        .map_err(|e| error!("Failed to clear old sent activities: {e}"))
        .ok();

      diesel::delete(
        received_activity::table.filter(received_activity::published.lt(now() - 3.months())),
      )
      .execute(&mut conn)
      .await
      .map(|_| info!("Done."))
      .map_err(|e| error!("Failed to clear old received activities: {e}"))
      .ok();
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
}

/// overwrite posts and comments 30d after deletion
async fn overwrite_deleted_posts_and_comments(pool: &mut DbPool<'_>) {
  info!("Overwriting deleted posts...");
  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
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
      .execute(&mut conn)
      .await
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
      .execute(&mut conn)
      .await
      .map(|_| {
        info!("Done.");
      })
      .map_err(|e| error!("Failed to overwrite deleted comments: {e}"))
      .ok();
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
}

/// Re-calculate the site and community active counts every 12 hours
async fn active_counts(pool: &mut DbPool<'_>) {
  info!("Updating active site and community aggregates ...");

  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
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
          .execute(&mut conn)
          .await
          .map_err(|e| error!("Failed to update site stats: {e}"))
          .ok();

        let update_community_stmt = format!("update community_aggregates ca set users_active_{} = mv.count_ from community_aggregates_activity('{}') mv where ca.community_id = mv.community_id_", i.1, i.0);
        sql_query(update_community_stmt)
          .execute(&mut conn)
          .await
          .map_err(|e| error!("Failed to update community stats: {e}"))
          .ok();
      }

      info!("Done.");
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
}

/// Set banned to false after ban expires
async fn update_banned_when_expired(pool: &mut DbPool<'_>) {
  info!("Updating banned column if it expires ...");
  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
      diesel::update(
        person::table
          .filter(person::banned.eq(true))
          .filter(person::ban_expires.lt(now().nullable())),
      )
      .set(person::banned.eq(false))
      .execute(&mut conn)
      .await
      .map_err(|e| error!("Failed to update person.banned when expires: {e}"))
      .ok();

      diesel::delete(
        community_person_ban::table.filter(community_person_ban::expires.lt(now().nullable())),
      )
      .execute(&mut conn)
      .await
      .map_err(|e| error!("Failed to remove community_ban expired rows: {e}"))
      .ok();
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
}

/// Updates the instance software and version
///
/// TODO: if instance has been dead for a long time, it should be checked less frequently
async fn update_instance_software(
  pool: &mut DbPool<'_>,
  client: &ClientWithMiddleware,
) -> LemmyResult<()> {
  info!("Updating instances software and versions...");
  let conn = get_conn(pool).await;

  match conn {
    Ok(mut conn) => {
      let instances = instance::table.get_results::<Instance>(&mut conn).await?;

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
        let form = match client.get(&node_info_url).send().await {
          Ok(res) if res.status().is_client_error() => {
            // Instance doesnt have nodeinfo but sent a response, consider it alive
            Some(default_form)
          }
          Ok(res) => match res.json::<NodeInfo>().await {
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
            .execute(&mut conn)
            .await?;
        }
      }
      info!("Finished updating instances software and versions...");
    }
    Err(e) => {
      error!("Failed to get connection from pool: {e}");
    }
  }
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
