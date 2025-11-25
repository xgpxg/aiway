mod ip_region_count;
mod state;

use crate::args::Args;
use clap::Parser;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start() -> anyhow::Result<()> {
    let args = Arc::new(Args::parse());
    let sched = JobScheduler::new().await?;

    // 心跳检测
    let update_heartbeat = Job::new_async("every 5 seconds", |_, _| {
        Box::pin(state::update_timeout_heartbeat_node())
    })?;
    sched.add(update_heartbeat).await?;

    // 按每个省份聚合调用次数
    let ip_region_count = Job::new_async("every 1 minutes", move |_, _| {
        Box::pin(ip_region_count::ip_region_count(args.clone()))
    })?;
    sched.add(ip_region_count).await?;

    // 省份聚合调用次数数据清理
    let ip_region_count_clean = Job::new_async("every 1 hours", move |_, _| {
        Box::pin(ip_region_count::clean())
    })?;
    sched.add(ip_region_count_clean).await?;

    sched.start().await?;

    Ok(())
}
