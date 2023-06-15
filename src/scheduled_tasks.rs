use clokwerk::{Scheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::{now, IntervalDsl},
  Connection,
  ExpressionMethods,
  QueryDsl,
};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  schema::{
    activity,
    comment_aggregates,
    community_aggregates,
    community_person_ban,
    instance,
    person,
    post_aggregates,
  },
  source::instance::{Instance, InstanceForm},
  utils::{functions::hot_rank, naive_now},
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

  let mut conn_1 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_2 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_3 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_4 = PgConnection::establish(&db_url).expect("could not establish connection");

  // Run on startup
  active_counts(&mut conn_1);
  update_hot_ranks(&mut conn_1, false);
  update_banned_when_expired(&mut conn_1);
  clear_old_activities(&mut conn_1);

  // Update active counts every hour
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    active_counts(&mut conn_1);
    update_banned_when_expired(&mut conn_1);
  });

  // Update hot ranks every 5 minutes
  scheduler.every(CTimeUnits::minutes(5)).run(move || {
    update_hot_ranks(&mut conn_2, true);
  });

  // Clear old activities every week
  scheduler.every(CTimeUnits::weeks(1)).run(move || {
    clear_old_activities(&mut conn_3);
  });

  // Remove old rate limit buckets after 1 to 2 hours of inactivity
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let hour = Duration::from_secs(3600);
    context_1.settings_updated_channel().remove_older_than(hour);
  });

  scheduler.every(CTimeUnits::days(1)).run(move || {
    update_instance_software(&mut conn_4, &user_agent);
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending();
    thread::sleep(Duration::from_millis(1000));
  }
}

/// Update the hot_rank columns for the aggregates tables
fn update_hot_ranks(conn: &mut PgConnection, last_week_only: bool) {
  let mut post_update = diesel::update(post_aggregates::table).into_boxed();
  let mut comment_update = diesel::update(comment_aggregates::table).into_boxed();
  let mut community_update = diesel::update(community_aggregates::table).into_boxed();

  // Only update for the last week of content
  if last_week_only {
    info!("Updating hot ranks for last week...");
    let last_week = now - diesel::dsl::IntervalDsl::weeks(1);

    post_update = post_update.filter(post_aggregates::published.gt(last_week));
    comment_update = comment_update.filter(comment_aggregates::published.gt(last_week));
    community_update = community_update.filter(community_aggregates::published.gt(last_week));
  } else {
    info!("Updating hot ranks for all history...");
  }

  match post_update
    .set((
      post_aggregates::hot_rank.eq(hot_rank(post_aggregates::score, post_aggregates::published)),
      post_aggregates::hot_rank_active.eq(hot_rank(
        post_aggregates::score,
        post_aggregates::newest_comment_time_necro,
      )),
    ))
    .execute(conn)
  {
    Ok(_) => {}
    Err(e) => {
      error!("Failed to update post_aggregates hot_ranks: {}", e)
    }
  }

  match comment_update
    .set(comment_aggregates::hot_rank.eq(hot_rank(
      comment_aggregates::score,
      comment_aggregates::published,
    )))
    .execute(conn)
  {
    Ok(_) => {}
    Err(e) => {
      error!("Failed to update comment_aggregates hot_ranks: {}", e)
    }
  }

  match community_update
    .set(community_aggregates::hot_rank.eq(hot_rank(
      community_aggregates::subscribers,
      community_aggregates::published,
    )))
    .execute(conn)
  {
    Ok(_) => {
      info!("Done.");
    }
    Err(e) => {
      error!("Failed to update community_aggregates hot_ranks: {}", e)
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
