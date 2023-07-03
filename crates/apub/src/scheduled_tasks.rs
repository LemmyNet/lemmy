use chrono::Duration;
use http::StatusCode;
use tracing::{error, info};
use url::Url;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::site::{Site, SiteUpdateForm};
use lemmy_db_schema::utils::naive_now;
use lemmy_utils::error::{LemmyError, LemmyResult};
use clokwerk::{AsyncScheduler, TimeUnits};
use lemmy_db_schema::traits::Crud;
use tokio::sync::OnceCell;

pub fn setup_federation_scheduled_tasks(context: LemmyContext) -> Result<(), LemmyError> {
    let mut scheduler = AsyncScheduler::new();

    // Check for dead federated instances
    static CONTEXT: OnceCell<LemmyContext> = OnceCell::const_new();
    CONTEXT.set(context).ok();
    scheduler.every(TimeUnits::minutes(1)).run(|| async {
        // TODO: this is not getting executed for some reason. change to daily once working
        check_dead_instances(CONTEXT.get().unwrap())
            .await
            .map_err(|e| error!("Failed to check federated instances: {e}"))
            .ok();
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

async fn check_dead_instances(context: &LemmyContext) -> LemmyResult<()> {
    info!("Checking if federated instances are alive");

    let day = Duration::days(1);
    let month = Duration::weeks(4);
    let now = naive_now();
    let instances: Vec<Site> = Site::read_remote_sites(context.pool()).await?.into_iter()
        // Dont need to check instances which were recently marked alive (eg by [ApubSite::from_json])
        .filter(|s| s.last_alive + day < now)
        // Dont check instances which have been dead for over a month
        .filter(|s| s.last_alive + month > now)
        .collect();

    for i in instances {
        let url: Url = i.actor_id.into();
        let res = context.client().get(url).send().await;
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

        let status = res.ok().map(|r| r.status());
        if status == Some(StatusCode::OK) {
            let form = SiteUpdateForm::builder().last_alive(Some(naive_now())).build();
            Site::update(context.pool(), i.id, &form).await?;
        }
    }
    info!("Finished checking if federated instances are alive");
    Ok(())
}