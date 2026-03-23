mod api_key_sync;
mod client;
mod config;
mod firewall;
mod global_plugin;
mod ip_region;
mod router;
mod servicer;

pub use api_key_sync::ApiKeySyncer;
pub use config::ConfigFactory;
pub use firewall::Firewalld;
pub use global_plugin::GlobalPluginFactory;
pub use ip_region::IpRegion;
pub use router::ROUTER;
pub use router::Router;
pub use servicer::Servicer;

const EAST_8_OFFSET: i32 = 8 * 3600;

pub fn display_time_with_timestamp_millis(ts: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ts)
        .expect("invalid timestamp")
        .with_timezone(&chrono::FixedOffset::east_opt(EAST_8_OFFSET).unwrap())
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
