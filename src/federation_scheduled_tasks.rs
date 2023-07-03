use std::sync::RwLock;
use chrono::Duration;
use tracing::{error, info};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::site::{SiteUpdateForm};
use lemmy_db_schema::utils::{get_conn, naive_now};
use lemmy_utils::error::{LemmyError, LemmyResult};
use clokwerk::{AsyncScheduler, TimeUnits};
use diesel::{Connection, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_db_schema::source::instance::{Instance, InstanceForm};
use lemmy_db_schema::traits::Crud;
use tokio::sync::OnceCell;
use lemmy_apub::DEAD_INSTANCES;
use lemmy_db_schema::schema::instance;
use lemmy_routes::nodeinfo::NodeInfo;

pub async fn setup_federation_scheduled_tasks(db_url: String, context: LemmyContext) -> Result<(), LemmyError> {
    let mut scheduler = AsyncScheduler::new();

    // Check for dead federated instances
    static CONTEXT: OnceCell<LemmyContext> = OnceCell::const_new();
    CONTEXT.set(context).ok();
    static DB_URL: OnceCell<String> = OnceCell::const_new();
    DB_URL.set(db_url).ok();
    scheduler.every(TimeUnits::minutes(1)).run(|| async {
        let mut conn = PgConnection::establish(DB_URL.get().unwrap()).expect("could not establish connection");
        // TODO: this is not getting executed for some reason. change to daily once working
        check_dead_instances(&mut conn, CONTEXT.get().unwrap())
            .await
            .map_err(|e| error!("Failed to check federated instances: {e}"))
            .ok();
    });

    // Mark instances which haven't been updated for three days or more as dead, and don't send
    // any activities to them.
    scheduler.every(TimeUnits::hour(1)).run(|| async {
        let mut conn = PgConnection::establish(DB_URL.get().unwrap()).expect("could not establish connection");
        update_dead_instances(&mut conn)
            .await;
    });

    // Manually run the scheduler in an event loop
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
    Ok(())
}

async fn update_dead_instances(conn: &mut PgConnection) {
    let day = Duration::days(3);
    let now = naive_now();
    let mut lock = DEAD_INSTANCES.write().unwrap();
    *lock = instance::table
        .select(instance::all_columns)
        .get_results::<Instance>(conn)
        .unwrap().into_iter()
        .filter(|i| i.updated() + day < now)
        .map(|i| i.domain)
        .collect();
}
async fn check_dead_instances(conn: &mut PgConnection, context: &LemmyContext) -> LemmyResult<()> {
    info!("Checking if federated instances are alive");

    let day = Duration::days(1);
    let month = Duration::weeks(4);
    let now = naive_now();
    let instances: Vec<Instance> = Instance::read_all(context.pool()).await?.into_iter()
        // Dont need to check instances which were recently marked alive (eg by [ApubSite::from_json])
        .filter(|s| s.updated() + day < now)
        // Dont check instances which have been dead for over a month
        .filter(|s| s.updated() + month > now)
        .collect();

    for instance in instances {
        // TODO: nodeinfo is not required for activitypub federation, so some alive instances
        //       may fail this check. in practice this might be irrelevant, not sure
        let node_info_url = format!("https://{}/nodeinfo/2.0.json", instance.domain);

        let res = context.client()
            .get(&node_info_url)
            .send().await;

        if let Ok(res) = res {
            if let Ok(node_info) = res.json::<NodeInfo>().await {
                let software = node_info.software.as_ref();
                let form = InstanceForm::builder()
                    .domain(instance.domain)
                    .software(software.and_then(|s| s.name.clone()))
                    .version(software.and_then(|s| s.version.clone()))
                    .updated(Some(naive_now()))
                    .build();

                diesel::update(instance::table.find(instance.id))
                    .set(form)
                    .get_result::<Instance>(conn).unwrap();
            }
        }
    }
    info!("Finished checking if federated instances are alive");
    Ok(())
}