use crate::VERSION;
use anyhow::bail;
use common::{app_dir, data_dir};
use rbatis::RBatis;
use rbs::Value;
use semver::{Version, VersionReq};
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use logging::log;

mod v0_1_0;

#[derive(EnumIter, Debug, PartialEq)]
pub enum AppVersion {
    V0_1_0,
}

impl FromStr for AppVersion {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.1.0" => Ok(AppVersion::V0_1_0),
            _ => bail!("Unknown version: {}", s),
        }
    }
}

impl Display for AppVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AppVersion::V0_1_0 => "0.1.0".to_string(),
        };
        write!(f, "{}", str)
    }
}


pub trait Migrator {
    #[allow(unused)]
    fn version(&self) -> AppVersion;
    async fn up(&self, conn: &mut RBatis) -> anyhow::Result<()>;
}

pub enum MigratorHandler {
    V0_1_0(v0_1_0::Handler),
}

impl MigratorHandler {
    pub async fn up(&self, conn: &mut RBatis) -> anyhow::Result<()> {
        match self {
            MigratorHandler::V0_1_0(handler) => handler.up(conn).await,
        }
    }
}

impl AppVersion {
    fn get_migrator(&self) -> MigratorHandler {
        match self {
            AppVersion::V0_1_0 => MigratorHandler::V0_1_0(v0_1_0::Handler),
        }
    }
}

const SELECT_VERSION_SQL: &str =
    "SELECT config_value FROM system_config where config_key = 'version'";
const INSERT_VERSION_SQL: &str =
    "INSERT INTO system_config (config_key, config_value) VALUES ('version', ?)";
const UPDATE_VERSION_SQL: &str =
    "UPDATE system_config SET config_value = ? WHERE config_key = 'version'";
pub async fn run(rb: &mut RBatis) {
    // 获取系统已有的版本号
    let result = rb
        .query_decode::<Option<String>>(SELECT_VERSION_SQL, vec![])
        .await;
    if let Err(e) = result {
        log::error!("Failed to get system version: {}", e);
        exit(1);
    }
    let result = result.unwrap();

    // 当前已安装版本
    let current_version = match result {
        Some(value) => value,
        None => {
            // 库中没有版本信息，设置已安装版本为当前程序版本
            rb.exec(INSERT_VERSION_SQL, vec![Value::String(VERSION.to_string())])
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to set system version: {}", e);
                    exit(1);
                });
            return;
        }
    };

    log::info!("Version: {}", current_version);

    // 版本号与当前版本一致，无需升级
    if current_version == VERSION {
        log::info!("Installed version is up to date, no upgrade required");
        return;
    }

    // 系统当前版本（已安装的版本）
    let current_version = Version::parse(&current_version).unwrap_or_else(|e| {
        log::error!("Failed to parse installed version: {}", e);
        exit(1);
    });

    // 待升级版本
    let app_version = Version::parse(VERSION).unwrap_or_else(|e| {
        log::error!("Failed to parse app version: {}", e);
        exit(1);
    });

    // 待升级版本必须大于已安装版本
    if app_version < current_version {
        log::error!(
            "Downgrading is not supported. Installed version: {}, current app version: {}",
            current_version,
            app_version
        );
        exit(1);
    }

    // 获取升级列表
    let req = VersionReq::parse(&format!(">{}, <={}", current_version, app_version)).unwrap();
    let mut versions = vec![];
    for item in AppVersion::iter() {
        //SAFE
        let version = Version::parse(item.to_string().as_str()).unwrap();
        if req.matches(&version) {
            versions.push(version);
        }
    }

    log::info!("Installing new version: {}", app_version);

    // 预解析语义化版本号为AppVersion，如有错误，提前失败
    let versions = versions
        .iter()
        .map(|v| AppVersion::from_str(&v.to_string()).unwrap())
        .collect::<Vec<_>>();

    // 备份数据文件夹
    if let Err(e) = backup_data_dir().await {
        log::error!("Failed to backup data dir: {}", e);
        exit(1);
    }

    let tx = match rb.acquire_begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            exit(1);
        }
    };

    // 执行变更
    for version in versions {
        let migrator = version.get_migrator();
        log::info!("Applying migration, version: {}", version);
        if let Err(e) = migrator.up(rb).await {
            log::error!("❌  Upgrade failed: {}", e);

            log::info!("Rolling back database");

            tx.rollback().await.unwrap_or_else(|e| {
                log::error!("Failed to rollback: {}", e);
            });

            // 回滚数据文件
            if let Err(e) = rollback_data_dir().await {
                log::error!("☠️  Failed to rollback data files: {}", e);
            }

            exit(1);
        }
    }

    // 数据库中更新版本号
    rb.exec(UPDATE_VERSION_SQL, vec![Value::String(VERSION.to_string())])
        .await
        .unwrap_or_else(|e| {
            log::error!(
                "NO!!! Upgrade finished, but failed to set system version: {}",
                e
            );
            exit(1);
        });

    log::info!("Upgrade to version {} success", app_version);
}

async fn backup_data_dir() -> anyhow::Result<()> {
    //let src = "data/";
    let src = &data_dir!().to_string_lossy().into_owned();
    //let dest = ".backup";
    let dest = &app_dir!(".backup").to_string_lossy().into_owned();

    log::info!("Backing up data dir");
    copy_data_dir(src, dest).await?;
    log::info!("Data dir backup complete");
    Ok(())
}
async fn rollback_data_dir() -> anyhow::Result<()> {
    //let src = ".backup/";
    let src = &app_dir!(".backup").to_string_lossy().into_owned();
    //let dest = "data";
    let dest = &data_dir!().to_string_lossy().into_owned();

    log::info!("Rolling back data dir");
    copy_data_dir(src, dest).await?;
    log::info!("Data dir rollback complete");

    Ok(())
}

async fn copy_data_dir(src: &str, dest: &str) -> anyhow::Result<()> {
    if !Path::new(src).exists() {
        bail!("Source directory does not exist: {}", src);
    }
    if !Path::new(dest).exists() {
        fs::create_dir_all(dest)?;
    }
    match copy_dir(src, dest).await {
        Ok(_) => {}
        Err(e) => {
            bail!("Failed to copy data dir: {}", e);
        }
    }

    Ok(())
}

async fn copy_dir(src: &str, dest: &str) -> anyhow::Result<()> {
    let options = fs_extra::dir::CopyOptions::new();
    let options = options.overwrite(true);
    fs_extra::dir::copy(src, dest, &options)?;
    Ok(())
}
