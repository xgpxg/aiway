mod state;

use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start() -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    let update_heartbeat = Job::new_async("every 5 seconds", |_, _| {
        Box::pin(state::update_timeout_heartbeat_node())
    })?;
    sched.add(update_heartbeat).await?;

    sched.start().await?;

    Ok(())
}
