//! # 通过IP查询地理位置
use crate::components::client::INNER_HTTP_CLIENT;
use std::process::exit;
use std::sync::OnceLock;

pub struct IpRegion {
    searcher: ip2region::Searcher,
}

static IP_REGION: OnceLock<IpRegion> = OnceLock::new();

impl IpRegion {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    async fn load() -> anyhow::Result<()> {
        let path = INNER_HTTP_CLIENT.fetch_ip_region_file().await?;
        log::info!("loaded ip region file, save to {}", path.display());
        let searcher = ip2region::Searcher::new(
            path.display().to_string(),
            ip2region::CachePolicy::FullMemory,
        )?;

        IP_REGION.get_or_init(|| IpRegion { searcher });
        Ok(())
    }

    #[allow(clippy::get_first)]
    pub fn search(ip: &str) -> (Option<String>, Option<String>, Option<String>) {
        let searcher = &IP_REGION.get().unwrap().searcher;
        let ip_region = searcher.search(ip).unwrap_or_default();
        let parts: Vec<&str> = ip_region.split('|').collect();
        let country = parts.get(0).map(|s| s.to_string());
        let province = parts.get(1).map(|s| s.to_string());
        let city = parts.get(2).map(|s| s.to_string());
        (country, province, city)
    }
}
