mod state;

use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start() -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    let sync_gateway_state = Job::new_async("every 10 seconds", |_, _| {
        Box::pin(state::sync_gateway_state())
    })?;
    sched.add(sync_gateway_state).await?;

    sched.start().await?;

    Ok(())
}
