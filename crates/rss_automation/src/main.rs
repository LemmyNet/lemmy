mod models;
mod processor;
mod scheduler;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Create and start the scheduler
    let scheduler = scheduler::FeedScheduler::new(pool);
    
    info!("Starting RSS feed automation service");
    if let Err(e) = scheduler.start().await {
        error!("Scheduler failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
