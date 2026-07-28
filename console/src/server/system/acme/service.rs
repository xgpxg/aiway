use super::AcmeConfig;
use super::request::AcmeConfigUpdateReq;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use certs::CertIssuer;
use common::data_dir;
use logging::log;
use std::fs;
use std::path::PathBuf;

/// 获取账户私钥文件路径
fn account_key_path(staging: bool) -> PathBuf {
    // 测试环境
    if staging {
        data_dir!("acme", "staging_account_key")
    } else {
        data_dir!("acme", "account_key")
    }
}

/// 确保 ACME 账户私钥已存在，返回私钥 PEM
///
/// 如果私钥文件不存在，创建新账户并持久化到文件
pub async fn ensure_account(config: &AcmeConfig) -> anyhow::Result<String> {
    let path = account_key_path(config.staging);

    if path.exists() {
        return Ok(fs::read_to_string(&path)?);
    }

    log::info!(
        "Creating new ACME account for email: {}, staging: {}",
        config.email,
        config.staging
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let issuer = CertIssuer::new(&config.email, config.staging).await?;
    let key = issuer.account_key_pem().to_string();
    fs::write(&path, &key)?;

    log::info!("ACME account key saved to: {:?}", path);
    Ok(key)
}

pub async fn update(req: AcmeConfigUpdateReq) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::Acme, &req.inner).await?;

    // 确保 ACME 账户私钥已存在
    if let Err(e) = ensure_account(&req.inner).await {
        log::warn!("Failed to create ACME account: {}", e);
    }

    Ok(())
}

pub async fn get() -> anyhow::Result<AcmeConfig> {
    let config: AcmeConfig = SystemConfig::get(ConfigKey::Acme).await?;
    Ok(config)
}

/// 获取原始配置（内部调用，不脱敏）
pub async fn get_raw() -> anyhow::Result<AcmeConfig> {
    SystemConfig::get(ConfigKey::Acme).await
}
