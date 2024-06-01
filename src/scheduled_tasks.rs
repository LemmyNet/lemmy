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
  source::{
    instance::{Instance, InstanceForm},
    local_user::LocalUser,
  },
  utils::{get_conn, naive_now, now, DbPool, DELETED_REPLACEMENT_TEXT},
};
use lemmy_routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use lemmy_utils::error::LemmyResult;
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;
use tracing::{error, info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: LemmyContext) -> LemmyResult<()> {
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
  // Daily tasks:
  // - Overwrite deleted & removed posts and comments every day
  // - Delete old denied users
  // - Update instance software
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let context = context_1.clone();

    async move {
      overwrite_deleted_posts_and_comments(&mut context.pool()).await;
      delete_old_denied_users(&mut context.pool()).await;
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
  delete_old_denied_users(pool).await;
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
        "comment",
        "a.hot_rank != 0",
        "SET hot_rank = r.hot_rank(a.score, a.published)",
      )
      .await;

      process_ranks_in_batches(
        &mut conn,
        "community",
        "a.hot_rank != 0",
        "SET hot_rank = r.hot_rank(a.subscribers, a.published)",
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
      r#"WITH batch AS (SELECT a.{id_column}
               FROM {aggregates_table} a
               WHERE a.published > $1 AND ({where_clause})
               ORDER BY a.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {aggregates_table} a {set_clause}
             FROM batch WHERE a.{id_column} = batch.{id_column} RETURNING a.published;
    "#,
      id_column = format!("{table_name}_id"),
      aggregates_table = format!("{table_name}_aggregates"),
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
      r#"WITH batch AS (SELECT pa.post_id
               FROM post_aggregates pa
               WHERE pa.published > $1
               AND (pa.hot_rank != 0 OR pa.hot_rank_active != 0)
               ORDER BY pa.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE post_aggregates pa
           SET hot_rank = r.hot_rank(pa.score, pa.published),
           hot_rank_active = r.hot_rank(pa.score, pa.newest_comment_time_necro),
           scaled_rank = r.scaled_rank(pa.score, pa.published, ca.users_active_month)
         FROM batch, community_aggregates ca
         WHERE pa.post_id = batch.post_id and pa.community_id = ca.community_id RETURNING pa.published;
    "#,
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
      diesel::delete(
        sent_activity::table.filter(sent_activity::published.lt(now() - IntervalDsl::days(7))),
      )
      .execute(&mut conn)
      .await
      .map_err(|e| error!("Failed to clear old sent activities: {e}"))
      .ok();

      diesel::delete(
        received_activity::table
          .filter(received_activity::published.lt(now() - IntervalDsl::days(7))),
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

async fn delete_old_denied_users(pool: &mut DbPool<'_>) {
  LocalUser::delete_old_denied_local_users(pool)
    .await
    .map(|_| {
      info!("Done.");
    })
    .map_err(|e| error!("Failed to deleted old denied users: {e}"))
    .ok();
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

/// Updates the instance software and version.
///
/// Does so using the /.well-known/nodeinfo protocol described here:
/// https://github.com/jhass/nodeinfo/blob/main/PROTOCOL.md
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
        if let Some(form) = build_update_instance_form(&instance.domain, client).await {
          Instance::update(pool, instance.id, form).await?;
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

/// This builds an instance update form, for a given domain.
/// If the instance sends a response, but doesn't have a well-known or nodeinfo,
/// Then return a default form with only the updated field.
///
/// TODO This function is a bit of a nightmare with its embedded matches, but the only other way
/// would be to extract the fetches into functions which return the default_form on errors.
async fn build_update_instance_form(
  domain: &str,
  client: &ClientWithMiddleware,
) -> Option<InstanceForm> {
  // The `updated` column is used to check if instances are alive. If it is more than three
  // days in the past, no outgoing activities will be sent to that instance. However
  // not every Fediverse instance has a valid Nodeinfo endpoint (its not required for
  // Activitypub). That's why we always need to mark instances as updated if they are
  // alive.
  let mut instance_form = InstanceForm::builder()
    .domain(domain.to_string())
    .updated(Some(naive_now()))
    .build();

  // First, fetch their /.well-known/nodeinfo, then extract the correct nodeinfo link from it
  let well_known_url = format!("https://{}/.well-known/nodeinfo", domain);

  match client.get(&well_known_url).send().await {
    Ok(res) if res.status().is_client_error() => {
      // Instance doesn't have well-known but sent a response, consider it alive
      Some(instance_form)
    }
    Ok(res) => match res.json::<NodeInfoWellKnown>().await {
      Ok(well_known) => {
        // Find the first link where the rel contains the allowed rels above
        match well_known.links.into_iter().find(|links| {
          links
            .rel
            .as_str()
            .starts_with("http://nodeinfo.diaspora.software/ns/schema/2.")
        }) {
          Some(well_known_link) => {
            let node_info_url = well_known_link.href;

            // Fetch the node_info from the well known href
            match client.get(node_info_url).send().await {
              Ok(node_info_res) => match node_info_res.json::<NodeInfo>().await {
                Ok(node_info) => {
                  // Instance sent valid nodeinfo, write it to db
                  // Set the instance form fields.
                  if let Some(software) = node_info.software.as_ref() {
                    instance_form.software.clone_from(&software.name);
                    instance_form.version.clone_from(&software.version);
                  }
                  Some(instance_form)
                }
                Err(_) => Some(instance_form),
              },
              Err(_) => Some(instance_form),
            }
          }
          // If none is found, use the default form above
          None => Some(instance_form),
        }
      }
      Err(_) => {
        // No valid nodeinfo but valid HTTP response, consider instance alive
        Some(instance_form)
      }
    },
    Err(_) => {
      // dead instance, do nothing
      None
    }
  }
}
#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::scheduled_tasks::build_update_instance_form;
  use lemmy_api_common::request::client_builder;
  use lemmy_utils::{error::LemmyResult, settings::structs::Settings, LemmyErrorType};
  use pretty_assertions::assert_eq;
  use reqwest_middleware::ClientBuilder;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_nodeinfo_voyager_lemmy_ml() -> LemmyResult<()> {
    let client = ClientBuilder::new(client_builder(&Settings::default()).build()?).build();
    let form = build_update_instance_form("voyager.lemmy.ml", &client)
      .await
      .ok_or(LemmyErrorType::CouldntFindObject)?;
    assert_eq!(
      form.software.ok_or(LemmyErrorType::CouldntFindObject)?,
      "lemmy"
    );
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_nodeinfo_mastodon_social() -> LemmyResult<()> {
    let client = ClientBuilder::new(client_builder(&Settings::default()).build()?).build();
    let form = build_update_instance_form("mastodon.social", &client)
      .await
      .ok_or(LemmyErrorType::CouldntFindObject)?;
    assert_eq!(
      form.software.ok_or(LemmyErrorType::CouldntFindObject)?,
      "mastodon"
    );
    Ok(())
  }
}
