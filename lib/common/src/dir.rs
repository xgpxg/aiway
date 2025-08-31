use std::path::PathBuf;
use std::sync::LazyLock;

/// 应用相关目录
pub struct AppDir {
    // 应用根目录，即可执行文件所在目录
    app_root_dir: PathBuf,
    // 资源目录
    resources_dir: PathBuf,
    // 数据目录
    data_dir: PathBuf,
    // 临时文件目录
    temp_dir: PathBuf,
    // 日志目录
    log_dir: PathBuf,
}
// 目录缓存
static DIR: LazyLock<AppDir> = LazyLock::new(|| AppDir::new());
impl AppDir {
    pub fn new() -> Self {
        let app_root = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let resources_dir = app_root.join("resources");
        let data_dir = app_root.join("data");
        let temp_dir = data_dir.join("temp");
        let log_dir = app_root.join("logs");
        AppDir {
            app_root_dir: app_root,
            resources_dir,
            temp_dir,
            data_dir,
            log_dir,
        }
    }
    /// 应用根目录
    pub fn app_root_dir() -> &'static PathBuf {
        &DIR.app_root_dir
    }

    /// 资源目录
    pub fn resources_dir() -> &'static PathBuf {
        &DIR.resources_dir
    }

    /// 数据目录
    pub fn data_dir() -> &'static PathBuf {
        &DIR.data_dir
    }

    /// 临时目录
    pub fn temp_dir() -> &'static PathBuf {
        &DIR.temp_dir
    }

    pub fn log_dir() -> &'static PathBuf {
        &DIR.log_dir
    }
}
#[macro_export]
macro_rules! app_dir {
    () => {
        $crate::dir::AppDir::app_root_dir()
    };

    ($($component: expr),+) => {
        {
            let mut path = $crate::dir::AppDir::app_root_dir().clone();
            $(
                path.push($component);
            )+
            path
        }
    };
}

#[macro_export]
macro_rules! resources_dir {
    () => {
        $crate::dir::AppDir::resources_dir()
    };

    ($($component: expr),+) => {
        {
            let mut path = $crate::dir::AppDir::resources_dir().clone();
            $(
                path.push($component);
            )+
            path
        }
    };
}

#[macro_export]
macro_rules! data_dir {
    () => {
        $crate::dir::AppDir::data_dir()
    };

    ($($component: expr),+) => {
        {
            let mut path = $crate::dir::AppDir::data_dir().clone();
            $(
                path.push($component);
            )+
            path
        }
    };
}

#[macro_export]
macro_rules! temp_dir {
    () => {
        $crate::dir::AppDir::temp_dir()
    };

    ($($component: expr),+) => {
        {
            let mut path = $crate::dir::AppDir::temp_dir().clone();
            $(
                path.push($component);
            )+
            path
        }
    };
}

#[macro_export]
macro_rules! file_dir {
    () => {
        $crate::dir::AppDir::data_dir().join("file")
    };

    ($($component: expr),+) => {
        {
            let mut path = $crate::dir::AppDir::data_dir().join("file");
            $(
                path.push($component);
            )+
            path
        }
    };
}
