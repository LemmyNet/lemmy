use clokwerk::{Scheduler, TimeUnits};
use diesel::{dsl::now, Connection, ExpressionMethods, QueryDsl};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use lemmy_db_schema::{
  schema::{comment_aggregates, community_aggregates, post_aggregates},
  utils::functions::hot_rank,
};
use lemmy_utils::error::LemmyError;
use std::{thread, time::Duration};
use tracing::info;

/// Schedules various cleanup tasks for lemmy in a background thread
pub fn setup(db_url: String) -> Result<(), LemmyError> {
  // Setup the connections
  let mut scheduler = Scheduler::new();

  let mut conn_1 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_2 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_3 = PgConnection::establish(&db_url).expect("could not establish connection");
  let mut conn_4 = PgConnection::establish(&db_url).expect("could not establish connection");

  active_counts(&mut conn_1);
  update_banned_when_expired(&mut conn_1);
  update_hot_ranks(&mut conn_1, false);
  clear_old_activities(&mut conn_1);

  scheduler.every(1.hour()).run(move || {
    active_counts(&mut conn_2);
    update_banned_when_expired(&mut conn_2);
  });
  // Clear old activities every week
  scheduler.every(TimeUnits::weeks(1)).run(move || {
    clear_old_activities(&mut conn_3);
  });
  scheduler.every(TimeUnits::minutes(5)).run(move || {
    update_hot_ranks(&mut conn_4, true);
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

  post_update
    .set((
      post_aggregates::hot_rank.eq(hot_rank(post_aggregates::score, post_aggregates::published)),
      post_aggregates::hot_rank_active.eq(hot_rank(
        post_aggregates::score,
        post_aggregates::newest_comment_time_necro,
      )),
    ))
    .execute(conn)
    .expect("update post_aggregate hot_ranks");

  comment_update
    .set(comment_aggregates::hot_rank.eq(hot_rank(
      comment_aggregates::score,
      comment_aggregates::published,
    )))
    .execute(conn)
    .expect("update comment_aggregate hot_ranks");

  community_update
    .set(community_aggregates::hot_rank.eq(hot_rank(
      community_aggregates::subscribers,
      community_aggregates::published,
    )))
    .execute(conn)
    .expect("update community_aggregate hot_ranks");
  info!("Done.");
}

/// Clear old activities (this table gets very large)
fn clear_old_activities(conn: &mut PgConnection) {
  use diesel::dsl::IntervalDsl;
  use lemmy_db_schema::schema::activity::dsl::{activity, published};
  info!("Clearing old activities...");
  diesel::delete(activity.filter(published.lt(now - 6.months())))
    .execute(conn)
    .expect("clear old activities");
  info!("Done.");
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
    sql_query(update_site_stmt)
      .execute(conn)
      .expect("update site stats");

    let update_community_stmt = format!("update community_aggregates ca set users_active_{} = mv.count_ from community_aggregates_activity('{}') mv where ca.community_id = mv.community_id_", i.1, i.0);
    sql_query(update_community_stmt)
      .execute(conn)
      .expect("update community stats");
  }

  info!("Done.");
}

/// Set banned to false after ban expires
fn update_banned_when_expired(conn: &mut PgConnection) {
  info!("Updating banned column if it expires ...");
  let update_ban_expires_stmt =
    "update person set banned = false where banned = true and ban_expires < now()";
  sql_query(update_ban_expires_stmt)
    .execute(conn)
    .expect("update banned when expires");
}
