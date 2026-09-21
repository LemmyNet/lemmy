use crate::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use activitypub_federation::config::Data;
use chrono::{DateTime, TimeZone, Utc};
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  QueryableByName,
  SelectableHelper,
  dsl::{IntervalDsl, count, count_star, exists, not, update},
  query_builder::AsQuery,
  sql_query,
  sql_types::{BigInt, Float8, Integer, Timestamptz},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_uplete::uplete;
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::plugin_hook_after,
  send_activity::{ActivityChannel, SendActivityData},
  utils::send_webmention,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    instance::{Instance, InstanceForm},
    local_user::LocalUser,
    post::{Post, PostUpdateForm},
  },
  utils::DELETED_REPLACEMENT_TEXT,
};
use lemmy_db_schema_file::schema::{
  comment,
  community,
  community_actions,
  federation_blocklist,
  instance,
  instance_actions,
  local_site,
  local_user,
  local_user_invite,
  person,
  post,
  received_activity,
  sent_activity,
  site,
};
use lemmy_db_views_site::{FederatedInstanceView, SiteView};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
  utils::{functions::coalesce, now},
};
use lemmy_utils::{
  DB_BATCH_SIZE,
  error::{LemmyErrorType, LemmyResult},
};
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: Data<LemmyContext>) -> LemmyResult<()> {
  // https://github.com/mdsherry/clokwerk/issues/38
  let mut scheduler = AsyncScheduler::with_tz(Utc);

  // Every 1 minute run plugin hooks
  scheduler.every(CTimeUnits::minutes(1)).run(async move || {
    plugin_hook_after("scheduled_task_1_min", &());
  });

  let context_1 = context.clone();
  // Every 10 minutes update hot ranks, delete expired captchas and publish scheduled posts
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.clone();

    async move {
      update_hot_ranks(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update hot ranks: {e}"))
        .ok();
      publish_scheduled_posts(&context)
        .await
        .inspect_err(|e| warn!("Failed to publish scheduled posts: {e}"))
        .ok();
      plugin_hook_after("scheduled_task_10_mins", &());
    }
  });

  let context_1 = context.clone();
  // Hourly tasks:
  // - Update active daily counts
  // - Expired bans
  // - Expired instance blocks
  // - Expired invitations
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let context = context_1.clone();

    async move {
      active_counts(&mut context.pool(), ONE_DAY)
        .await
        .inspect_err(|e| warn!("Failed to update active counts: {e}"))
        .ok();
      update_banned_when_expired(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update expired bans: {e}"))
        .ok();
      delete_instance_block_when_expired(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete expired instance bans: {e}"))
        .ok();
      delete_invitations_when_expired(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete expired invitations: {e}"))
        .ok();
      plugin_hook_after("scheduled_task_1_hour", &());
    }
  });

  let context_1 = context.reset_request_count();
  // Daily tasks:
  // - Update site and community activity counts
  // - Update local user count
  // - Update linked instance count
  // - Update total counts (posts, comments, users, communities)
  // - Update user retention percents
  // - Update language usage percents
  // - Update banned user percent
  // - Overwrite deleted & removed posts and comments every day
  // - Delete old denied users
  // - Update instance software
  // - Delete old outgoing activities
  scheduler.every(CTimeUnits::days(1)).run(move || {
    let context = context_1.reset_request_count();

    async move {
      all_active_counts(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update active counts: {e}"))
        .ok();
      update_local_user_count(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update local user count: {e}"))
        .ok();
      update_linked_instance_count(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update linked instance count: {e}"))
        .ok();
      update_total_counts(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update total counts: {e}"))
        .ok();
      all_user_retention_percents(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update user retention percents: {e}"))
        .ok();
      update_language_usage_percents(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update language usage breakdown: {e}"))
        .ok();
      update_banned_user_percent(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update banner user percent: {e}"))
        .ok();
      overwrite_deleted_posts_and_comments(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to overwrite deleted posts/comments: {e}"))
        .ok();
      delete_old_denied_users(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete old denied users: {e}"))
        .ok();
      update_instance_software(&mut context.pool(), &context)
        .await
        .inspect_err(|e| warn!("Failed to update instance software: {e}"))
        .ok();
      clear_old_activities(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to clear old activities: {e}"))
        .ok();
      plugin_hook_after("scheduled_task_daily", &());
    }
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
async fn update_hot_ranks(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating hot ranks for all history...");

  let conn = &mut get_conn(pool).await?;

  process_post_aggregates_ranks_in_batches(conn).await?;

  process_ranks_in_batches(
    conn,
    "comment",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.score, a.published_at)",
  )
  .await?;

  process_ranks_in_batches(
    conn,
    "community",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.subscribers, a.published_at)",
  )
  .await?;

  info!("Finished hot ranks update!");
  Ok(())
}

#[derive(QueryableByName)]
struct HotRanksUpdateResult {
  #[diesel(sql_type = Timestamptz)]
  published_at: DateTime<Utc>,
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
) -> LemmyResult<()> {
  let process_start_time: DateTime<Utc> = Utc.timestamp_opt(0, 0).single().unwrap_or_default();

  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    // Raw `sql_query` is used as a performance optimization - Diesel does not support doing this
    // in a single query (neither as a CTE, nor using a subquery)
    let updated_rows = sql_query(format!(
      r#"WITH batch AS (SELECT a.id
               FROM {table_name} a
               WHERE a.published_at > $1 AND ({where_clause})
               ORDER BY a.published_at
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {table_name} a {set_clause}
             FROM batch WHERE a.id = batch.id RETURNING a.published_at;
    "#,
    ))
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<BigInt, _>(DB_BATCH_SIZE)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      LemmyErrorType::Unknown(format!("Failed to update {} hot_ranks: {}", table_name, e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published_at);
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    table_name, processed_rows_count
  );
  Ok(())
}

/// Post aggregates is a special case, since it needs to join to the community_aggregates
/// table, to get the active monthly user counts.
async fn process_post_aggregates_ranks_in_batches(conn: &mut AsyncPgConnection) -> LemmyResult<()> {
  let process_start_time: DateTime<Utc> = Utc.timestamp_opt(0, 0).single().unwrap_or_default();

  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    let updated_rows = sql_query(
      r#"WITH batch AS (SELECT pa.id
           FROM post pa
           WHERE pa.published_at > $1
           AND (pa.hot_rank != 0 OR pa.hot_rank_active != 0)
           ORDER BY pa.published_at
           LIMIT $2
           FOR UPDATE SKIP LOCKED)
      UPDATE post pa
      SET hot_rank = r.hot_rank(pa.score, pa.published_at),
          hot_rank_active = r.hot_rank(pa.score, coalesce(pa.newest_comment_time_necro_at, pa.published_at)),
          scaled_rank = r.scaled_rank(pa.score, pa.published_at, ca.interactions_month)
      FROM batch, community ca
      WHERE pa.id = batch.id
      AND pa.community_id = ca.id
      RETURNING pa.published_at;
"#,
    )
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<BigInt, _>(DB_BATCH_SIZE)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      LemmyErrorType::Unknown(format!("Failed to update post_aggregates hot_ranks: {}", e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published_at);
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    "post_aggregates", processed_rows_count
  );
  Ok(())
}

/// Clear old activities (this table gets very large)
async fn clear_old_activities(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Clearing old activities...");
  let conn = &mut get_conn(pool).await?;

  diesel::delete(
    sent_activity::table.filter(sent_activity::published_at.lt(now() - IntervalDsl::days(7))),
  )
  .execute(conn)
  .await?;

  diesel::delete(
    received_activity::table
      .filter(received_activity::published_at.lt(now() - IntervalDsl::days(7))),
  )
  .execute(conn)
  .await?;
  info!("Done.");
  Ok(())
}

async fn delete_old_denied_users(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  LocalUser::delete_old_denied_local_users(pool).await?;
  info!("Done.");
  Ok(())
}

/// overwrite posts and comments 30d after deletion
async fn overwrite_deleted_posts_and_comments(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Overwriting deleted posts...");
  let conn = &mut get_conn(pool).await?;

  diesel::update(
    post::table
      .filter(post::deleted.eq(true))
      .filter(post::updated_at.lt(now().nullable() - 1.months()))
      .filter(post::body.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set((
    post::body.eq(DELETED_REPLACEMENT_TEXT),
    post::name.eq(DELETED_REPLACEMENT_TEXT),
  ))
  .execute(conn)
  .await?;

  info!("Overwriting deleted comments...");
  diesel::update(
    comment::table
      .filter(comment::deleted.eq(true))
      .filter(comment::updated_at.lt(now().nullable() - 1.months()))
      .filter(comment::content.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set(comment::content.eq(DELETED_REPLACEMENT_TEXT))
  .execute(conn)
  .await?;
  info!("Done.");
  Ok(())
}

const ONE_DAY: (&str, &str) = ("1 day", "day");
const ONE_WEEK: (&str, &str) = ("1 week", "week");
const ONE_MONTH: (&str, &str) = ("1 month", "month");
const SIX_MONTHS: (&str, &str) = ("6 months", "half_year");

const ALL_ACTIVE_INTERVALS: [(&str, &str); 4] = [ONE_DAY, ONE_WEEK, ONE_MONTH, SIX_MONTHS];

const ALL_RETENTION_INTERVALS: [(&str, &str); 2] = [ONE_MONTH, SIX_MONTHS];

#[derive(QueryableByName)]
struct SiteActivitySelectResult {
  #[diesel(sql_type = Integer)]
  site_aggregates_activity: i32,
}

#[derive(QueryableByName)]
struct UserRetentionSelectResult {
  #[diesel(sql_type = Integer)]
  previous_count: i32,
  #[diesel(sql_type = Integer)]
  retained_count: i32,
}

#[derive(QueryableByName)]
struct CommunityAggregatesUpdateResult {
  #[diesel(sql_type = Integer)]
  community_id: i32,
}

/// Re-calculate the site and community active counts for a given interval
async fn active_counts(pool: &mut DbPool<'_>, interval: (&str, &str)) -> LemmyResult<()> {
  info!(
    "Updating active site and community aggregates for {}...",
    interval.0
  );

  let conn = &mut get_conn(pool).await?;
  process_site_aggregates(conn, interval).await?;
  process_community_aggregates(
    conn,
    interval,
    "users_active",
    "community_aggregates_activity",
  )
  .await?;

  Ok(())
}

/// Re-calculate all the active counts
async fn all_active_counts(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  for i in ALL_ACTIVE_INTERVALS {
    active_counts(pool, i).await?;
  }
  let conn = &mut get_conn(pool).await?;
  process_community_aggregates(
    conn,
    ONE_MONTH,
    "interactions",
    "community_aggregates_interactions",
  )
  .await?;
  Ok(())
}

/// Re-calculate the user retention percents
async fn all_user_retention_percents(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;

  for i in ALL_RETENTION_INTERVALS {
    process_retention_percents(conn, i).await?;
  }

  Ok(())
}

async fn process_site_aggregates(
  conn: &mut AsyncPgConnection,
  interval: (&str, &str),
) -> LemmyResult<()> {
  // Select the site count result first
  let site_activity = sql_query(format!(
    "select * from r.site_aggregates_activity('{}')",
    interval.0
  ))
  .get_result::<SiteActivitySelectResult>(conn)
  .await
  .inspect_err(|e| warn!("Failed to fetch site activity: {e}"))?;

  let processed_rows = site_activity.site_aggregates_activity;

  // Update the site count
  sql_query(format!(
    "update local_site set users_active_{} = $1",
    interval.1,
  ))
  .bind::<Integer, _>(processed_rows)
  .execute(conn)
  .await
  .inspect_err(|e| warn!("Failed to update site stats: {e}"))
  .ok();

  info!(
    "Finished site_aggregates active_{} (processed {} rows)",
    interval.1, processed_rows
  );

  Ok(())
}

/// Find local, non-bot users active in the period of `interval` length preceding the current
/// one, then work out what percentage of them are also active in the current period.
async fn process_retention_percents(
  conn: &mut AsyncPgConnection,
  interval: (&str, &str),
) -> LemmyResult<()> {
  let retention = sql_query(format!(
    r#"WITH previous_active AS (
         SELECT c.creator_id AS person_id FROM comment c
           INNER JOIN person pe ON pe.id = c.creator_id
           WHERE c.published_at >= (CURRENT_DATE - INTERVAL '{i}' * 2)
             AND c.published_at < (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT p.creator_id FROM post p
           INNER JOIN person pe ON pe.id = p.creator_id
           WHERE p.published_at >= (CURRENT_DATE - INTERVAL '{i}' * 2)
             AND p.published_at < (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT pa.person_id FROM post_actions pa
           INNER JOIN person pe ON pe.id = pa.person_id
           WHERE pa.voted_at >= (CURRENT_DATE - INTERVAL '{i}' * 2)
             AND pa.voted_at < (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT ca.person_id FROM comment_actions ca
           INNER JOIN person pe ON pe.id = ca.person_id
           WHERE ca.voted_at >= (CURRENT_DATE - INTERVAL '{i}' * 2)
             AND ca.voted_at < (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
       ),
       current_active AS (
         SELECT c.creator_id AS person_id FROM comment c
           INNER JOIN person pe ON pe.id = c.creator_id
           WHERE c.published_at >= (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT p.creator_id FROM post p
           INNER JOIN person pe ON pe.id = p.creator_id
           WHERE p.published_at >= (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT pa.person_id FROM post_actions pa
           INNER JOIN person pe ON pe.id = pa.person_id
           WHERE pa.voted_at >= (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
         UNION
         SELECT ca.person_id FROM comment_actions ca
           INNER JOIN person pe ON pe.id = ca.person_id
           WHERE ca.voted_at >= (CURRENT_DATE - INTERVAL '{i}')
             AND pe.local = TRUE AND pe.bot_account = FALSE
       )
       SELECT
         (SELECT count(*) FROM previous_active)::integer AS previous_count,
         (SELECT count(*) FROM previous_active
            INNER JOIN current_active USING (person_id))::integer AS retained_count"#,
    i = interval.0
  ))
  .get_result::<UserRetentionSelectResult>(conn)
  .await
  .inspect_err(|e| warn!("Failed to calculate user retention: {e}"))?;

  let percent = if retention.previous_count == 0 {
    0.0
  } else {
    f64::from(retention.retained_count) / f64::from(retention.previous_count) * 100.0
  };

  sql_query(format!(
    "update local_site set user_retention_{}_percent = $1",
    interval.1,
  ))
  .bind::<Float8, _>(percent)
  .execute(conn)
  .await
  .inspect_err(|e| warn!("Failed to update user retention stats: {e}"))
  .ok();

  info!(
    "Finished user retention_{} ({} out of {} retained)",
    interval.1, retention.retained_count, retention.previous_count
  );

  Ok(())
}

async fn process_community_aggregates(
  conn: &mut AsyncPgConnection,
  interval: (&str, &str),
  field_name_prefix: &str,
  function_name: &str,
) -> LemmyResult<()> {
  // Select the community count results into a temp table.
  let caggs_temp_table = &format!("community_aggregates_temp_table_{}", interval.1);

  // Drop temp table before and after, just in case
  let drop_caggs_temp_table = &format!("DROP TABLE IF EXISTS {caggs_temp_table}");
  sql_query(drop_caggs_temp_table).execute(conn).await.ok();

  sql_query(format!(
    "CREATE TEMP TABLE {caggs_temp_table} AS SELECT * FROM r.{function_name}('{}')",
    interval.0
  ))
  .execute(conn)
  .await
  .inspect_err(|e| warn!("Failed to create temp community_aggregates table: {e}"))?;

  // Split up into 1000 community transaction batches
  let update_batch_size = 1000;
  let mut processed_rows_count = 0;
  let mut prev_community_id_res = Some(0);

  while let Some(prev_community_id) = prev_community_id_res {
    let updated_rows = sql_query(format!(
      "UPDATE community a
            SET {field_name_prefix}_{} = b.count_
            FROM (
              SELECT count_, community_id_
              FROM {caggs_temp_table}
              WHERE community_id_ > $1
              ORDER BY community_id_
              LIMIT $2
            ) AS b
            WHERE a.id = b.community_id_
            RETURNING a.id AS community_id
            ",
      interval.1
    ))
    .bind::<Integer, _>(prev_community_id)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<CommunityAggregatesUpdateResult>(conn)
    .await
    .inspect_err(|e| warn!("Failed to update community stats: {e}"))?;

    processed_rows_count += updated_rows.len();
    prev_community_id_res = updated_rows.last().map(|row| row.community_id);
  }

  // Dead communities are absent in the temporary table.
  // Reset the activity counter of the dead communities
  sql_query(format!("UPDATE community a SET {field_name_prefix}_{} = 0 WHERE a.id NOT IN (SELECT DISTINCT b.community_id_ FROM {caggs_temp_table} b)",
      interval.1))
    .execute(conn)
    .await
    .inspect_err(|e| warn!("Failed to zero-out community stats: {e}"))
    .ok();

  // Drop the temp table just in case
  sql_query(drop_caggs_temp_table).execute(conn).await.ok();

  info!(
    "Finished community_aggregates {field_name_prefix}_{} (processed {} rows)",
    interval.1, processed_rows_count
  );

  info!("Done.");
  Ok(())
}

/// Approved, non-deleted local users, left joined to their ban action on the local instance.
#[diesel::dsl::auto_type]
fn approved_local_users() -> _ {
  local_user::table
    .inner_join(
      person::table.left_join(
        instance_actions::table
          .inner_join(instance::table.inner_join(site::table.inner_join(local_site::table))),
      ),
    )
    // only count approved users
    .filter(local_user::accepted_application)
    // ignore deleted accounts
    .filter(not(person::deleted))
}

async fn update_local_user_count(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating the local user count...");

  let conn = &mut get_conn(pool).await?;
  let user_count = approved_local_users()
    // ignore banned accounts
    .filter(instance_actions::received_ban_at.is_null())
    .select(count(local_user::id))
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  update(local_site::table)
    .set(local_site::local_users.eq(user_count))
    .execute(conn)
    .await?;

  info!("Done.");
  Ok(())
}

async fn update_linked_instance_count(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating the linked instance count...");

  let linked_instance_count = FederatedInstanceView::count(pool).await?;

  let conn = &mut get_conn(pool).await?;

  update(local_site::table)
    .set(local_site::linked_instances.eq(linked_instance_count))
    .execute(conn)
    .await?;

  info!("Done.");
  Ok(())
}

async fn update_total_counts(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating total counts ...");

  let conn = &mut get_conn(pool).await?;

  let total_post_count = post::table
    .filter(not(post::deleted.or(post::removed)))
    .select(count_star())
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  let total_comment_count = comment::table
    .filter(not(comment::deleted.or(comment::removed)))
    .select(count_star())
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  let total_community_count = community::table
    .filter(not(community::deleted.or(community::removed)))
    .select(count_star())
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  let banned_on_home_instance = instance_actions::table
    .find((person::id, person::instance_id))
    .filter(instance_actions::received_ban_at.is_not_null());

  let total_user_count = person::table
    .filter(not(person::deleted))
    .filter(not(exists(banned_on_home_instance)))
    .select(count_star())
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  update(local_site::table)
    .set((
      local_site::total_posts.eq(total_post_count),
      local_site::total_comments.eq(total_comment_count),
      local_site::total_users.eq(total_user_count),
      local_site::total_communities.eq(total_community_count),
    ))
    .execute(conn)
    .await?;

  info!("Done.");

  Ok(())
}
/// Update each language with its percentage share of local posts and local comments
async fn update_language_usage_percents(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Calculating local language usage percentages ...");

  let conn = &mut get_conn(pool).await?;

  sql_query(
    r#"
    WITH post_counts AS (
      SELECT language_id, COUNT(*) AS n FROM post WHERE local GROUP BY language_id
    ),
    comment_counts AS (
      SELECT language_id, COUNT(*) AS n FROM comment WHERE local GROUP BY language_id
    ),
    totals AS (
      SELECT local_posts, local_comments FROM local_site LIMIT 1
    )
    UPDATE language l SET
      usage_in_local_posts = COALESCE(pc.n::float8 / NULLIF(t.local_posts, 0) * 100.0, 0), /* set usage_in_local_posts to 0 if t.local_posts is 0*/ 
      usage_in_local_comments = COALESCE(cc.n::float8 / NULLIF(t.local_comments, 0) * 100.0, 0)
    FROM language l2
      CROSS JOIN totals t
      LEFT JOIN post_counts pc ON pc.language_id = l2.id
      LEFT JOIN comment_counts cc ON cc.language_id = l2.id
    WHERE l.id = l2.id
    "#,
  )
  .execute(conn)
  .await?;

  info!("Done.");
  Ok(())
}

/// Set banned to false after ban expires
async fn update_banned_when_expired(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating banned column if it expires ...");
  let conn = &mut get_conn(pool).await?;

  uplete(community_actions::table.filter(community_actions::ban_expires_at.lt(now().nullable())))
    .set_null(community_actions::received_ban_at)
    .set_null(community_actions::ban_expires_at)
    .as_query()
    .execute(conn)
    .await?;

  uplete(instance_actions::table.filter(instance_actions::ban_expires_at.lt(now().nullable())))
    .set_null(instance_actions::received_ban_at)
    .set_null(instance_actions::ban_expires_at)
    .as_query()
    .execute(conn)
    .await?;
  Ok(())
}

/// Set banned to false after ban expires
async fn delete_instance_block_when_expired(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Delete instance blocks when expired ...");
  let conn = &mut get_conn(pool).await?;

  diesel::delete(
    federation_blocklist::table.filter(federation_blocklist::expires_at.lt(now().nullable())),
  )
  .execute(conn)
  .await?;
  Ok(())
}

/// Set invitations to Expired
async fn delete_invitations_when_expired(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;
  diesel::delete(
    local_user_invite::table.filter(local_user_invite::expires_at.lt(now().nullable())),
  )
  .execute(conn)
  .await?;
  Ok(())
}

/// Find all unpublished posts with scheduled date in the future, and publish them.
async fn publish_scheduled_posts(context: &Data<LemmyContext>) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let local_instance_id = SiteView::read_local(pool).await?.instance.id;
  let conn = &mut get_conn(pool).await?;

  let not_community_banned_action = community_actions::table
    .find((person::id, community::id))
    .filter(community_actions::received_ban_at.is_not_null());

  let not_local_banned_action = instance_actions::table
    .find((person::id, local_instance_id))
    .filter(instance_actions::received_ban_at.is_not_null());

  let scheduled_posts: Vec<_> = post::table
    .inner_join(community::table)
    .inner_join(person::table)
    // find all posts which have scheduled_publish_time that is in the  past
    .filter(post::scheduled_publish_time_at.is_not_null())
    .filter(coalesce(post::scheduled_publish_time_at, now()).lt(now()))
    // make sure the post, person and community are still around
    .filter(not(post::deleted.or(post::removed)))
    .filter(not(person::deleted))
    .filter(not(community::removed.or(community::deleted)))
    // ensure that user isnt banned from community
    .filter(not(exists(not_community_banned_action)))
    // ensure that user isnt banned from local
    .filter(not(exists(not_local_banned_action)))
    .select((Post::as_select(), Community::as_select()))
    .get_results::<(Post, Community)>(conn)
    .await?;

  for (post, community) in scheduled_posts {
    // mark post as published in db
    let form = PostUpdateForm {
      scheduled_publish_time_at: Some(None),
      ..Default::default()
    };
    Post::update(&mut context.pool(), post.id, &form).await?;

    // send out post via federation and webmention
    let send_activity = SendActivityData::CreatePost(post.clone());
    ActivityChannel::submit_activity(send_activity, context)?;
    send_webmention(post, &community, context.clone());
  }
  Ok(())
}

/// Updates the instance software and version.
///
/// Does so using the /.well-known/nodeinfo protocol described here:
/// https://github.com/jhass/nodeinfo/blob/main/PROTOCOL.md
///
/// TODO: if instance has been dead for a long time, it should be checked less frequently
async fn update_instance_software(
  pool: &mut DbPool<'_>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  info!("Updating instances software and versions...");
  let conn = &mut get_conn(pool).await?;

  let instances = instance::table.get_results::<Instance>(conn).await?;

  for instance in instances {
    if let Some(form) = build_update_instance_form(&instance.domain, context).await {
      Instance::update(pool, instance.id, form).await?;
    }
  }
  info!("Finished updating instances software and versions...");
  Ok(())
}

/// This builds an instance update form, for a given domain.
/// If the instance sends a response, but doesn't have a well-known or nodeinfo,
/// Then return a default form with only the updated field.
async fn build_update_instance_form(
  domain: &str,
  context: &Data<LemmyContext>,
) -> Option<InstanceForm> {
  // The `updated` column is used to check if instances are alive. If it is more than three
  // days in the past, no outgoing activities will be sent to that instance. However
  // not every Fediverse instance has a valid Nodeinfo endpoint (its not required for
  // Activitypub). That's why we always need to mark instances as updated if they are
  // alive.
  let mut instance_form = InstanceForm {
    updated_at: Some(Utc::now()),
    ..InstanceForm::new(domain.to_string())
  };

  // First, fetch their /.well-known/nodeinfo, then extract the correct nodeinfo link from it
  let well_known_url = Url::parse(&format!("https://{}/.well-known/nodeinfo", domain)).ok()?;
  context.is_valid_ip(&well_known_url).await.ok()?;

  let Ok(res) = context.client().get(well_known_url).send().await else {
    // This is the only kind of error that means the instance is dead
    return None;
  };
  let status = res.status();
  if status.is_client_error() || status.is_server_error() {
    return None;
  }

  // In this block, returning `None` is ignored, and only means not writing nodeinfo to db
  async {
    let node_info_url = res
      .json::<NodeInfoWellKnown>()
      .await
      .ok()?
      .links
      .into_iter()
      .find(|links| {
        links
          .rel
          .as_str()
          .starts_with("http://nodeinfo.diaspora.software/ns/schema/2.")
      })?
      .href;

    let software = context
      .client()
      .get(node_info_url)
      .send()
      .await
      .ok()?
      .json::<NodeInfo>()
      .await
      .ok()?
      .software?;

    instance_form.software = software.name;
    instance_form.version = software.version;

    Some(())
  }
  .await;

  Some(instance_form)
}

async fn update_banned_user_percent(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating the banned rate...");

  let conn = &mut get_conn(pool).await?;

  let total_local_user_count = approved_local_users()
    .select(count(local_user::id))
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  let banned_local_user_count = approved_local_users()
    // only count banned accounts
    .filter(instance_actions::received_ban_at.is_not_null())
    .select(count(local_user::id))
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  let ban_rate = if total_local_user_count == 0 {
    0.0
  } else {
    f64::from(banned_local_user_count) / f64::from(total_local_user_count) * 100.0
  };

  update(local_site::table)
    .set(local_site::ban_rate.eq(ban_rate))
    .execute(conn)
    .await?;

  info!(
    "Finished ban_rate ({banned_local_user_count} out of {total_local_user_count} local users banned, {ban_rate:.2}%)"
  );
  Ok(())
}

#[cfg(test)]
mod tests {

  use super::*;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      language::Language,
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    test_data::TestData,
    traits::Likeable,
  };
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  async fn test_nodeinfo_lemmy_ml() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let form = build_update_instance_form("lemmy.ml", &context)
      .await
      .ok_or(LemmyErrorType::NotFound)?;
    assert_eq!(form.software.ok_or(LemmyErrorType::NotFound)?, "lemmy");
    Ok(())
  }

  #[tokio::test]
  async fn test_nodeinfo_mastodon_social() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let form = build_update_instance_form("mastodon.social", &context)
      .await
      .ok_or(LemmyErrorType::NotFound)?;
    assert_eq!(form.software.ok_or(LemmyErrorType::NotFound)?, "mastodon");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_scheduled_tasks() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let data = TestData::create(pool).await?;
    let community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "name".to_owned(), "pubkey".to_owned()),
    )
    .await?;
    let person = Person::create(
      pool,
      &PersonInsertForm::new("felicity".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;
    let post = Post::create(
      pool,
      &PostInsertForm::new("i am grrreat".to_owned(), person.id, community.id),
    )
    .await?;
    PostActions::like(pool, &PostLikeForm::new(post.id, person.id, Some(true))).await?;

    active_counts(pool, ONE_DAY).await?;
    all_active_counts(pool).await?;
    update_local_user_count(pool).await?;
    update_hot_ranks(pool).await?;
    update_banned_when_expired(pool).await?;
    delete_instance_block_when_expired(pool).await?;
    clear_old_activities(pool).await?;
    overwrite_deleted_posts_and_comments(pool).await?;
    delete_old_denied_users(pool).await?;
    update_instance_software(pool, &context).await?;
    publish_scheduled_posts(&context).await?;

    let community_after = Community::read(pool, community.id).await?;
    assert_eq!(
      community_after,
      Community {
        posts: 1,
        users_active_day: 1,
        users_active_week: 1,
        users_active_month: 1,
        users_active_half_year: 1,
        interactions_month: 1,
        ..community_after.clone()
      }
    );

    data.delete(pool).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_dead_community_active_counts_reset_to_zero() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let public_key = "public_key".to_owned();
    let data = TestData::create(pool).await?;
    let person = Person::create(
      pool,
      &PersonInsertForm::new("user".to_owned(), "public_key".to_owned(), data.instance.id),
    )
    .await?;

    let dead_community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "dead".to_owned(), public_key.clone()),
    )
    .await?;
    let two_years_ago = Utc::now() - chrono::Duration::days(730);
    Post::create(
      pool,
      &PostInsertForm {
        published_at: Some(two_years_ago),
        ..PostInsertForm::new("dead post".to_owned(), person.id, dead_community.id)
      },
    )
    .await?;

    let alive_community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "alive".to_owned(), public_key.clone()),
    )
    .await?;
    Post::create(
      pool,
      &PostInsertForm::new("alive post".to_owned(), person.id, alive_community.id),
    )
    .await?;

    all_active_counts(pool).await?;

    let dead_stats = Community::read(pool, dead_community.id).await?;
    assert_eq!(
      dead_stats.users_active_day, 0,
      "dead: users_active_day should be 0"
    );
    assert_eq!(
      dead_stats.users_active_week, 0,
      "dead: users_active_week should be 0"
    );
    assert_eq!(
      dead_stats.users_active_month, 0,
      "dead: users_active_month should be 0"
    );
    assert_eq!(
      dead_stats.users_active_half_year, 0,
      "dead: users_active_half_year should be 0"
    );

    let alive_stats = Community::read(pool, alive_community.id).await?;
    assert_eq!(
      alive_stats.users_active_day, 1,
      "alive: users_active_day should be 1"
    );
    assert_eq!(
      alive_stats.users_active_week, 1,
      "alive: users_active_week should be 1"
    );
    assert_eq!(
      alive_stats.users_active_month, 1,
      "alive: users_active_month should be 1"
    );
    assert_eq!(
      alive_stats.users_active_half_year, 1,
      "alive: users_active_half_year should be 1"
    );

    data.delete(pool).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_update_total_counts() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    // Setup local site
    let data = TestData::create(pool).await?;
    // insert local community and user
    let local_community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "local".to_owned(), "pubkey".to_owned()),
    )
    .await?;
    let local_person = Person::create(
      pool,
      &PersonInsertForm::new("felicity".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;

    // insert linked instance, with a user and a community
    let instance0 = Instance::read_or_create(pool, "example0.com").await?;
    Instance::read_or_create(pool, "example1.com").await?;
    Community::create(
      pool,
      &CommunityInsertForm::new(instance0.id, "remote".to_owned(), "pubkey".to_owned()),
    )
    .await?;
    let remote_person = Person::create(
      pool,
      &PersonInsertForm::new("remote".to_owned(), "pubkey".to_owned(), instance0.id),
    )
    .await?;

    // local user posts in the local community and comments on it
    let local_post = Post::create(
      pool,
      &PostInsertForm::new("local post".to_owned(), local_person.id, local_community.id),
    )
    .await?;
    let local_comment = CommentInsertForm::new(
      local_person.id,
      local_post.id,
      local_community.id,
      "local".into(),
    );
    Comment::create(pool, &local_comment, None).await?;

    // remote user comments on the local post
    let remote_comment = CommentInsertForm::new(
      remote_person.id,
      local_post.id,
      local_community.id,
      "remote".into(),
    );
    Comment::create(pool, &remote_comment, None).await?;

    // remote user posts in the local community
    let remote_post = PostInsertForm::new(
      "remote post".to_owned(),
      remote_person.id,
      local_community.id,
    );
    Post::create(pool, &remote_post).await?;

    let local_site_before = SiteView::read_local(pool).await?.local_site;
    assert_eq!(0, local_site_before.total_posts);
    assert_eq!(0, local_site_before.total_comments);
    assert_eq!(0, local_site_before.total_users);
    assert_eq!(0, local_site_before.total_communities);
    assert_eq!(0, local_site_before.linked_instances);

    // run the queries
    update_total_counts(pool).await?;
    update_linked_instance_count(pool).await?;
    let local_site_after = SiteView::read_local(pool).await?.local_site;

    // totals include both local and federated objects
    assert_eq!(2, local_site_after.total_posts);
    assert_eq!(2, local_site_after.total_comments);
    assert_eq!(4, local_site_after.total_users);
    assert_eq!(2, local_site_after.total_communities);
    assert_eq!(2, local_site_after.linked_instances);

    data.delete(pool).await?;
    Instance::delete_all(pool).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_update_language_usage_percents() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let data = TestData::create(pool).await?;
    let community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "name".to_owned(), "pubkey".to_owned()),
    )
    .await?;
    let person = Person::create(
      pool,
      &PersonInsertForm::new("felicity".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;

    let en_id = Language::read_id_from_code(pool, "en").await?;
    let de_id = Language::read_id_from_code(pool, "de").await?;

    // Create 2 English posts and 1 German post (expect 67% and 33%)
    for _ in 0..2 {
      Post::create(
        pool,
        &PostInsertForm {
          language_id: Some(en_id),
          ..PostInsertForm::new("english post".to_owned(), person.id, community.id)
        },
      )
      .await?;
    }
    let post = Post::create(
      pool,
      &PostInsertForm {
        language_id: Some(de_id),
        ..PostInsertForm::new("german post".to_owned(), person.id, community.id)
      },
    )
    .await?;

    // Create 1 English comment and 3 German comments (expect 25% and 75%)
    Comment::create(
      pool,
      &CommentInsertForm {
        language_id: Some(en_id),
        ..CommentInsertForm::new(
          person.id,
          post.id,
          community.id,
          "english comment".to_owned(),
        )
      },
      None,
    )
    .await?;
    for _ in 0..3 {
      Comment::create(
        pool,
        &CommentInsertForm {
          language_id: Some(de_id),
          ..CommentInsertForm::new(
            person.id,
            post.id,
            community.id,
            "german comment".to_owned(),
          )
        },
        None,
      )
      .await?;
    }

    update_language_usage_percents(pool).await?;

    let en_language = Language::read_from_id(pool, en_id).await?;
    let de_language = Language::read_from_id(pool, de_id).await?;

    assert_eq!(
      (en_language.usage_in_local_posts * 100.0).round() / 100.0,
      66.67
    );
    assert_eq!(
      (de_language.usage_in_local_posts * 100.0).round() / 100.0,
      33.33
    );
    assert_eq!(
      (en_language.usage_in_local_comments * 100.0).round() / 100.0,
      25.0
    );
    assert_eq!(
      (de_language.usage_in_local_comments * 100.0).round() / 100.0,
      75.0
    );

    data.delete(pool).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_process_user_retentions() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let data = TestData::create(pool).await?;
    let community = Community::create(
      pool,
      &CommunityInsertForm::new(data.instance.id, "name".to_owned(), "pubkey".to_owned()),
    )
    .await?;

    let retained_person = Person::create(
      pool,
      &PersonInsertForm::new("retained".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;
    let churned_person = Person::create(
      pool,
      &PersonInsertForm::new("churned".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;
    let new_person = Person::create(
      pool,
      &PersonInsertForm::new("newcomer".to_owned(), "pubkey".to_owned(), data.instance.id),
    )
    .await?;

    let now = Utc::now();
    // Active ~45 days ago (previous month window) and ~10 days ago (current month window)
    Post::create(
      pool,
      &PostInsertForm {
        published_at: Some(now - chrono::Duration::days(45)),
        ..PostInsertForm::new(
          "retained post 1".to_owned(),
          retained_person.id,
          community.id,
        )
      },
    )
    .await?;
    Post::create(
      pool,
      &PostInsertForm {
        published_at: Some(now - chrono::Duration::days(10)),
        ..PostInsertForm::new(
          "retained post 2".to_owned(),
          retained_person.id,
          community.id,
        )
      },
    )
    .await?;
    // Active only ~45 days ago, churned by the current window
    Post::create(
      pool,
      &PostInsertForm {
        published_at: Some(now - chrono::Duration::days(45)),
        ..PostInsertForm::new("churned post".to_owned(), churned_person.id, community.id)
      },
    )
    .await?;
    // Active only ~10 days ago, wasn't active in the previous window
    Post::create(
      pool,
      &PostInsertForm {
        published_at: Some(now - chrono::Duration::days(10)),
        ..PostInsertForm::new("new post".to_owned(), new_person.id, community.id)
      },
    )
    .await?;

    let conn = &mut get_conn(pool).await?;
    process_retention_percents(conn, ONE_MONTH).await?;

    let local_site = SiteView::read_local(pool).await?.local_site;
    assert_eq!(local_site.user_retention_month_percent, 50.0);

    data.delete(pool).await?;
    Ok(())
  }
}
