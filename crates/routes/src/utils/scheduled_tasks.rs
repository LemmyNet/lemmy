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
  dsl::{IntervalDsl, count, exists, not, update},
  query_builder::AsQuery,
  sql_query,
  sql_types::{BigInt, Integer, Timestamptz},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_uplete::uplete;
use lemmy_api_utils::{
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
  utils::{DELETED_REPLACEMENT_TEXT, DbPool, functions::coalesce, get_conn, now},
};
use lemmy_db_schema_file::schema::{
  captcha_answer,
  comment,
  community,
  community_actions,
  federation_blocklist,
  instance,
  instance_actions,
  local_site,
  local_user,
  person,
  post,
  received_activity,
  sent_activity,
  site,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  DB_BATCH_SIZE,
  error::{LemmyErrorType, LemmyResult},
};
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;
use tracing::{info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: Data<LemmyContext>) -> LemmyResult<()> {
  // https://github.com/mdsherry/clokwerk/issues/38
  let mut scheduler = AsyncScheduler::with_tz(Utc);

  let context_1 = context.clone();
  // Every 10 minutes update hot ranks, delete expired captchas and publish scheduled posts
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.clone();

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
  // Hourly tasks:
  // - Update active daily counts
  // - Expired bans
  // - Expired instance blocks
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
    }
  });

  let context_1 = context.reset_request_count();
  // Daily tasks:
  // - Update site and community activity counts
  // - Update local user count
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
      clear_old_activities(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to clear old activities: {e}"))
        .ok();
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

async fn delete_expired_captcha_answers(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;

  diesel::delete(
    captcha_answer::table.filter(captcha_answer::published_at.lt(now() - IntervalDsl::minutes(10))),
  )
  .execute(conn)
  .await?;
  info!("Done.");

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

#[derive(QueryableByName)]
struct SiteActivitySelectResult {
  #[diesel(sql_type = Integer)]
  site_aggregates_activity: i32,
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

  // Drop the temp table just in case
  sql_query(drop_caggs_temp_table).execute(conn).await.ok();

  info!(
    "Finished community_aggregates {field_name_prefix}_{} (processed {} rows)",
    interval.1, processed_rows_count
  );

  info!("Done.");
  Ok(())
}

async fn update_local_user_count(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  info!("Updating the local user count...");

  let conn = &mut get_conn(pool).await?;
  let user_count = local_user::table
    .inner_join(
      person::table.left_join(
        instance_actions::table
          .inner_join(instance::table.inner_join(site::table.inner_join(local_site::table))),
      ),
    )
    // only count approved users
    .filter(local_user::accepted_application)
    // ignore banned and deleted accounts
    .filter(instance_actions::received_ban_at.is_null())
    .filter(not(person::deleted))
    .select(count(local_user::id))
    .first::<i64>(conn)
    .await
    .map(i32::try_from)??;

  update(local_site::table)
    .set(local_site::users.eq(user_count))
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
  let conn = &mut get_conn(pool).await?;

  let instances = instance::table.get_results::<Instance>(conn).await?;

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
    updated_at: Some(Utc::now()),
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
  use lemmy_api_utils::request::client_builder;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    test_data::TestData,
    traits::{Crud, Likeable},
  };
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
  async fn test_scheduled_tasks() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let data = TestData::create(pool).await?;
    let community = Community::create(
      pool,
      &CommunityInsertForm::new(
        data.instance.id,
        "name".to_owned(),
        "title".to_owned(),
        "pubkey".to_owned(),
      ),
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
    PostActions::like(pool, &PostLikeForm::new(post.id, person.id, true)).await?;

    active_counts(pool, ONE_DAY).await?;
    all_active_counts(pool).await?;
    update_local_user_count(pool).await?;
    update_hot_ranks(pool).await?;
    update_banned_when_expired(pool).await?;
    delete_instance_block_when_expired(pool).await?;
    clear_old_activities(pool).await?;
    overwrite_deleted_posts_and_comments(pool).await?;
    delete_old_denied_users(pool).await?;
    update_instance_software(pool, context.client()).await?;
    delete_expired_captcha_answers(pool).await?;
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
}
