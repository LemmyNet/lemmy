use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use diesel::{Connection, ExpressionMethods, QueryDsl};
use lemmy_db_schema::{
  source::instance::{Instance, InstanceForm},
  utils::naive_now,
};
use lemmy_routes::nodeinfo::NodeInfo;
use lemmy_utils::{error::LemmyError, REQWEST_TIMEOUT};
use reqwest::blocking::Client;
use std::{thread, time::Duration};
use tracing::info;

/// Schedules various cleanup tasks for lemmy in a background thread
pub fn setup(db_url: String, user_agent: String) -> Result<(), LemmyError> {
  // Setup the connections
  let mut scheduler = Scheduler::new();

  let mut conn = PgConnection::establish(&db_url).expect("could not establish connection");

  let mut conn_2 = PgConnection::establish(&db_url).expect("could not establish connection");

  active_counts(&mut conn);
  update_banned_when_expired(&mut conn);

  // On startup, reindex the tables non-concurrently
  // TODO remove this for now, since it slows down startup a lot on lemmy.ml
  reindex_aggregates_tables(&mut conn, true);
  scheduler.every(1.hour()).run(move || {
    let conn = &mut PgConnection::establish(&db_url)
      .unwrap_or_else(|_| panic!("Error connecting to {db_url}"));
    active_counts(conn);
    update_banned_when_expired(conn);
    reindex_aggregates_tables(conn, true);
    drop_ccnew_indexes(conn);
  });

  clear_old_activities(&mut conn);
  scheduler.every(1.weeks()).run(move || {
    clear_old_activities(&mut conn);
  });

  update_instance_software(&mut conn_2, &user_agent);
  scheduler.every(1.days()).run(move || {
    update_instance_software(&mut conn_2, &user_agent);
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
fn reindex_aggregates_tables(conn: &mut PgConnection, concurrently: bool) {
  for table_name in &[
    "post_aggregates",
    "comment_aggregates",
    "community_aggregates",
  ] {
    reindex_table(conn, table_name, concurrently);
  }
}

fn reindex_table(conn: &mut PgConnection, table_name: &str, concurrently: bool) {
  let concurrently_str = if concurrently { "concurrently" } else { "" };
  info!("Reindexing table {} {} ...", concurrently_str, table_name);
  let query = format!("reindex table {concurrently_str} {table_name}");
  sql_query(query).execute(conn).expect("reindex table");
  info!("Done.");
}

/// Clear old activities (this table gets very large)
fn clear_old_activities(conn: &mut PgConnection) {
  use diesel::dsl::{now, IntervalDsl};
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

/// Drops the phantom CCNEW indexes created by postgres
/// https://github.com/LemmyNet/lemmy/issues/2431
fn drop_ccnew_indexes(conn: &mut PgConnection) {
  info!("Dropping phantom ccnew indexes...");
  let drop_stmt = "select drop_ccnew_indexes()";
  sql_query(drop_stmt)
    .execute(conn)
    .expect("drop ccnew indexes");
}

/// Updates the instance software and version
fn update_instance_software(conn: &mut PgConnection, user_agent: &str) {
  use lemmy_db_schema::schema::instance;
  info!("Updating instances software and versions...");

  let client = Client::builder()
    .user_agent(user_agent)
    .timeout(REQWEST_TIMEOUT)
    .build()
    .expect("couldnt build reqwest client");

  let instances = instance::table
    .get_results::<Instance>(conn)
    .expect("no instances found");

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

      diesel::update(instance::table.find(instance.id))
        .set(form)
        .execute(conn)
        .expect("update site instance software");
    }
  }
  info!("Done.");
}

#[cfg(test)]
mod tests {
  use lemmy_routes::nodeinfo::NodeInfo;
  use reqwest::Client;

  #[tokio::test]
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
