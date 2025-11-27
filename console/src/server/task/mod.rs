mod ip_region_count;
mod request_status_count;
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
    let args_clone = args.clone();
    let ip_region_count = Job::new_async("every 1 minutes", move |_, _| {
        Box::pin(ip_region_count::ip_region_count(args_clone.clone()))
    })?;
    sched.add(ip_region_count).await?;

    // 省份聚合调用次数数据清理
    let ip_region_count_clean = Job::new_async("every 1 hours", move |_, _| {
        Box::pin(ip_region_count::clean())
    })?;
    sched.add(ip_region_count_clean).await?;

    // 请求状态统计聚合
    let args_clone = args.clone();
    let request_status_count = Job::new_async("every 1 minutes", move |_, _| {
        Box::pin(request_status_count::request_status_count(
            args_clone.clone(),
        ))
    })?;
    sched.add(request_status_count).await?;

    // 状态统计数据清理
    let request_status_count_clean = Job::new_async("every 1 hours", move |_, _| {
        Box::pin(request_status_count::clean())
    })?;
    sched.add(request_status_count_clean).await?;

    sched.start().await?;

    Ok(())
}
