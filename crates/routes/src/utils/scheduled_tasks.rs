use crate::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use activitypub_federation::config::Data;
use chrono::{DateTime, TimeZone, Utc};
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::{exists, not, IntervalDsl},
  query_builder::AsQuery,
  sql_query,
  sql_types::{Integer, Timestamptz},
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  QueryableByName,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::{
  context::LemmyContext,
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
  traits::Crud,
  utils::{functions::coalesce, get_conn, now, uplete, DbPool, DELETED_REPLACEMENT_TEXT},
};
use lemmy_db_schema_file::schema::{
  captcha_answer,
  comment,
  community,
  community_actions,
  federation_blocklist,
  instance,
  instance_actions,
  person,
  post,
  received_activity,
  sent_activity,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;
use tracing::{info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: Data<LemmyContext>) -> LemmyResult<()> {
  // Setup the connections
  let mut scheduler = AsyncScheduler::new();
  startup_jobs(&mut context.pool())
    .await
    .inspect_err(|e| warn!("Failed to run startup tasks: {e}"))
    .ok();

  let context_1 = context.clone();
  // Update active counts expired bans and unpublished posts every hour
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let context = context_1.clone();

    async move {
      active_counts(&mut context.pool())
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
    }
  });

  let context_1 = context.reset_request_count();
  // Every 10 minutes update hot ranks, delete expired captchas and publish scheduled posts
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.reset_request_count();

    async move {
      update_hot_ranks(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update hot ranks: {e}"))
        .ok();
      delete_expired_captcha_answers(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete expired captcha answers: {e}"))
        .ok();
      publish_scheduled_posts(&context)
        .await
        .inspect_err(|e| warn!("Failed to publish scheduled posts: {e}"))
        .ok();
    }
  });

  let context_1 = context.clone();
  // Clear old activities every week
  scheduler.every(CTimeUnits::weeks(1)).run(move || {
    let context = context_1.clone();

    async move {
      clear_old_activities(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to clear old activities: {e}"))
        .ok();
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
      overwrite_deleted_posts_and_comments(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to overwrite deleted posts/comments: {e}"))
        .ok();
      delete_old_denied_users(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete old denied users: {e}"))
        .ok();
      update_instance_software(&mut context.pool(), context.client())
        .await
        .inspect_err(|e| warn!("Failed to update instance software: {e}"))
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
async fn startup_jobs(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  active_counts(pool).await?;
  update_hot_ranks(pool).await?;
  update_banned_when_expired(pool).await?;
  delete_instance_block_when_expired(pool).await?;
  clear_old_activities(pool).await?;
  overwrite_deleted_posts_and_comments(pool).await?;
  delete_old_denied_users(pool).await?;
  Ok(())
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
async fn update_hot_ranks(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating hot ranks for all history...");

  let mut conn = get_conn(pool).await?;

  process_post_aggregates_ranks_in_batches(&mut conn).await?;

  process_ranks_in_batches(
    &mut conn,
    "comment",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.score, a.published)",
  )
  .await?;

  process_ranks_in_batches(
    &mut conn,
    "community",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.subscribers, a.published)",
  )
  .await?;

  info!("Finished hot ranks update!");
  Ok(())
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
) -> LemmyResult<()> {
  let process_start_time: DateTime<Utc> = Utc.timestamp_opt(0, 0).single().unwrap_or_default();

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    // Raw `sql_query` is used as a performance optimization - Diesel does not support doing this
    // in a single query (neither as a CTE, nor using a subquery)
    let updated_rows = sql_query(format!(
      r#"WITH batch AS (SELECT a.id
               FROM {table_name} a
               WHERE a.published > $1 AND ({where_clause})
               ORDER BY a.published
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {table_name} a {set_clause}
             FROM batch WHERE a.id = batch.id RETURNING a.published;
    "#,
    ))
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      LemmyErrorType::Unknown(format!("Failed to update {} hot_ranks: {}", table_name, e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published);
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

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    let updated_rows = sql_query(
      r#"WITH batch AS (SELECT pa.id
           FROM post pa
           WHERE pa.published > $1
           AND (pa.hot_rank != 0 OR pa.hot_rank_active != 0)
           ORDER BY pa.published
           LIMIT $2
           FOR UPDATE SKIP LOCKED)
      UPDATE post pa
      SET hot_rank = r.hot_rank(pa.score, pa.published),
          hot_rank_active = r.hot_rank(pa.score, pa.newest_comment_time_necro),
          scaled_rank = r.scaled_rank(pa.score, pa.published, ca.interactions_month)
      FROM batch, community ca
      WHERE pa.id = batch.id
      AND pa.community_id = ca.id
      RETURNING pa.published;
"#,
    )
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      LemmyErrorType::Unknown(format!("Failed to update post_aggregates hot_ranks: {}", e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published);
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    "post_aggregates", processed_rows_count
  );
  Ok(())
}

async fn delete_expired_captcha_answers(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let mut conn = get_conn(pool).await?;

  diesel::delete(
    captcha_answer::table.filter(captcha_answer::published.lt(now() - IntervalDsl::minutes(10))),
  )
  .execute(&mut conn)
  .await?;
  info!("Done.");

  Ok(())
}

/// Clear old activities (this table gets very large)
async fn clear_old_activities(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Clearing old activities...");
  let mut conn = get_conn(pool).await?;

  diesel::delete(
    sent_activity::table.filter(sent_activity::published.lt(now() - IntervalDsl::days(7))),
  )
  .execute(&mut conn)
  .await?;

  diesel::delete(
    received_activity::table.filter(received_activity::published.lt(now() - IntervalDsl::days(7))),
  )
  .execute(&mut conn)
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
  let mut conn = get_conn(pool).await?;

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
  .await?;

  info!("Overwriting deleted comments...");
  diesel::update(
    comment::table
      .filter(comment::deleted.eq(true))
      .filter(comment::updated.lt(now().nullable() - 1.months()))
      .filter(comment::content.ne(DELETED_REPLACEMENT_TEXT)),
  )
  .set(comment::content.eq(DELETED_REPLACEMENT_TEXT))
  .execute(&mut conn)
  .await?;
  info!("Done.");
  Ok(())
}

/// Re-calculate the site and community active counts every 12 hours
async fn active_counts(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating active site and community aggregates ...");

  let mut conn = get_conn(pool).await?;

  let intervals = vec![
    ("1 day", "day"),
    ("1 week", "week"),
    ("1 month", "month"),
    ("6 months", "half_year"),
  ];

  for (full_form, abbr) in &intervals {
    let update_site_stmt = format!(
      "update local_site set users_active_{} = (select r.site_aggregates_activity('{}')) where site_id = 1",
      abbr, full_form
    );
    sql_query(update_site_stmt).execute(&mut conn).await?;

    let update_community_stmt = format!("update community ca set users_active_{} = mv.count_ from r.community_aggregates_activity('{}') mv where ca.id = mv.community_id_", abbr, full_form);
    sql_query(update_community_stmt).execute(&mut conn).await?;
  }

  let update_interactions_stmt = "update community ca set interactions_month = mv.count_ from r.community_aggregates_interactions('1 month') mv where ca.id = mv.community_id_";
  sql_query(update_interactions_stmt)
    .execute(&mut conn)
    .await?;

  info!("Done.");
  Ok(())
}

/// Set banned to false after ban expires
async fn update_banned_when_expired(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating banned column if it expires ...");
  let mut conn = get_conn(pool).await?;

  uplete::new(community_actions::table.filter(community_actions::ban_expires.lt(now().nullable())))
    .set_null(community_actions::received_ban)
    .set_null(community_actions::ban_expires)
    .as_query()
    .execute(&mut conn)
    .await?;

  uplete::new(instance_actions::table.filter(instance_actions::ban_expires.lt(now().nullable())))
    .set_null(instance_actions::received_ban)
    .set_null(instance_actions::ban_expires)
    .as_query()
    .execute(&mut conn)
    .await?;
  Ok(())
}

/// Set banned to false after ban expires
async fn delete_instance_block_when_expired(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Delete instance blocks when expired ...");
  let mut conn = get_conn(pool).await?;

  diesel::delete(
    federation_blocklist::table.filter(federation_blocklist::expires.lt(now().nullable())),
  )
  .execute(&mut conn)
  .await?;
  Ok(())
}

/// Find all unpublished posts with scheduled date in the future, and publish them.
async fn publish_scheduled_posts(context: &Data<LemmyContext>) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let local_instance_id = SiteView::read_local(pool).await?.instance.id;
  let mut conn = get_conn(pool).await?;

  let not_community_banned_action = community_actions::table
    .find((person::id, community::id))
    .filter(community_actions::received_ban.is_not_null());

  let not_local_banned_action = instance_actions::table
    .find((person::id, local_instance_id))
    .filter(instance_actions::received_ban.is_not_null());

  let scheduled_posts: Vec<_> = post::table
    .inner_join(community::table)
    .inner_join(person::table)
    // find all posts which have scheduled_publish_time that is in the  past
    .filter(post::scheduled_publish_time.is_not_null())
    .filter(coalesce(post::scheduled_publish_time, now()).lt(now()))
    // make sure the post, person and community are still around
    .filter(not(post::deleted.or(post::removed)))
    .filter(not(person::deleted))
    .filter(not(community::removed.or(community::deleted)))
    // ensure that user isnt banned from community
    .filter(not(exists(not_community_banned_action)))
    // ensure that user isnt banned from local
    .filter(not(exists(not_local_banned_action)))
    .select((post::all_columns, community::all_columns))
    .get_results::<(Post, Community)>(&mut conn)
    .await?;

  for (post, community) in scheduled_posts {
    // mark post as published in db
    let form = PostUpdateForm {
      scheduled_publish_time: Some(None),
      ..Default::default()
    };
    Post::update(&mut context.pool(), post.id, &form).await?;

    // send out post via federation and webmention
    let send_activity = SendActivityData::CreatePost(post.clone());
    ActivityChannel::submit_activity(send_activity, context)?;
    send_webmention(post, &community);
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
  client: &ClientWithMiddleware,
) -> LemmyResult<()> {
  info!("Updating instances software and versions...");
  let mut conn = get_conn(pool).await?;

  let instances = instance::table.get_results::<Instance>(&mut conn).await?;

  for instance in instances {
    if let Some(form) = build_update_instance_form(&instance.domain, client).await {
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
  client: &ClientWithMiddleware,
) -> Option<InstanceForm> {
  // The `updated` column is used to check if instances are alive. If it is more than three
  // days in the past, no outgoing activities will be sent to that instance. However
  // not every Fediverse instance has a valid Nodeinfo endpoint (its not required for
  // Activitypub). That's why we always need to mark instances as updated if they are
  // alive.
  let mut instance_form = InstanceForm {
    updated: Some(Utc::now()),
    ..InstanceForm::new(domain.to_string())
  };

  // First, fetch their /.well-known/nodeinfo, then extract the correct nodeinfo link from it
  let well_known_url = format!("https://{}/.well-known/nodeinfo", domain);

  let Ok(res) = client.get(&well_known_url).send().await else {
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

    let software = client
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

#[cfg(test)]
mod tests {

  use super::*;
  use lemmy_api_common::request::client_builder;
  use lemmy_db_views_site::impls::create_test_instance;
  use lemmy_utils::{
    error::{LemmyErrorType, LemmyResult},
    settings::structs::Settings,
  };
  use pretty_assertions::assert_eq;
  use reqwest_middleware::ClientBuilder;
  use serial_test::serial;

  #[tokio::test]
  async fn test_nodeinfo_lemmy_ml() -> LemmyResult<()> {
    let client = ClientBuilder::new(client_builder(&Settings::default()).build()?).build();
    let form = build_update_instance_form("lemmy.ml", &client)
      .await
      .ok_or(LemmyErrorType::NotFound)?;
    assert_eq!(form.software.ok_or(LemmyErrorType::NotFound)?, "lemmy");
    Ok(())
  }

  #[tokio::test]
  async fn test_nodeinfo_mastodon_social() -> LemmyResult<()> {
    let client = ClientBuilder::new(client_builder(&Settings::default()).build()?).build();
    let form = build_update_instance_form("mastodon.social", &client)
      .await
      .ok_or(LemmyErrorType::NotFound)?;
    assert_eq!(form.software.ok_or(LemmyErrorType::NotFound)?, "mastodon");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_scheduled_tasks_no_errors() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let instance = create_test_instance(&mut context.pool()).await?;

    startup_jobs(&mut context.pool()).await?;
    update_instance_software(&mut context.pool(), context.client()).await?;
    delete_expired_captcha_answers(&mut context.pool()).await?;
    publish_scheduled_posts(&context).await?;
    Instance::delete(&mut context.pool(), instance.id).await?;
    Ok(())
  }
}
