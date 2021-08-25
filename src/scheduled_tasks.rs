// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::source::activity::Activity;
use log::info;
use std::{thread, time::Duration};

/// Schedules various cleanup tasks for lemmy in a background thread
pub fn setup(pool: DbPool) {
  let mut scheduler = Scheduler::new();

  let conn = pool.get().unwrap();
  active_counts(&conn);

  // On startup, reindex the tables non-concurrently
  reindex_aggregates_tables(&conn, false);
  scheduler.every(1.hour()).run(move || {
    active_counts(&conn);
    reindex_aggregates_tables(&conn, true);
  });

  let conn = pool.get().unwrap();
  clear_old_activities(&conn);
  scheduler.every(1.weeks()).run(move || {
    clear_old_activities(&conn);
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending();
    thread::sleep(Duration::from_millis(1000));
  }
}

/// Reindex the aggregates tables every one hour
/// This is necessary because hot_rank is actually a mutable function:
/// https://dba.stackexchange.com/questions/284052/how-to-create-an-index-based-on-a-time-based-function-in-postgres?noredirect=1#comment555727_284052
fn reindex_aggregates_tables(conn: &PgConnection, concurrently: bool) {
  for table_name in &[
    "post_aggregates",
    "comment_aggregates",
    "community_aggregates",
  ] {
    reindex_table(conn, table_name, concurrently);
  }
}

fn reindex_table(conn: &PgConnection, table_name: &str, concurrently: bool) {
  let concurrently_str = if concurrently { "concurrently" } else { "" };
  info!("Reindexing table {} {} ...", concurrently_str, table_name);
  let query = format!("reindex table {} {}", concurrently_str, table_name);
  sql_query(query).execute(conn).expect("reindex table");
  info!("Done.");
}

/// Clear old activities (this table gets very large)
fn clear_old_activities(conn: &PgConnection) {
  info!("Clearing old activities...");
  Activity::delete_olds(conn).expect("clear old activities");
  info!("Done.");
}

/// Re-calculate the site and community active counts every 12 hours
fn active_counts(conn: &PgConnection) {
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
