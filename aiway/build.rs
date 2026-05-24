use std::ffi::OsString;
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

fn main() {
    // 项目目录
    let project_dir = get_project_root().unwrap();

    // 二进制文件目录，需要提前编译console、gateway和logg
    let out_dir = env::var("OUT_DIR").unwrap_or_default();
    let release_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // 检测目标二进制文件是否存在，不存在则跳过（如 cargo check 场景）
    let gateway_bin = release_dir.join("gateway");
    if !gateway_bin.exists() {
        println!("cargo:warning=跳过二进制文件复制，目标文件尚未编译");
        return;
    }

    // 嵌入的二进制文件目录
    let bin_dir = project_dir.join("aiway/bin");
    fs::create_dir_all(&bin_dir).unwrap();

    // 复制二进制文件
    fs::copy(release_dir.join("gateway"), &bin_dir.join("gateway")).unwrap();
    fs::copy(release_dir.join("console"), &bin_dir.join("console")).unwrap();
    fs::copy(release_dir.join("logg"), &bin_dir.join("logg")).unwrap();
    fs::copy(release_dir.join("access"), &bin_dir.join("access")).unwrap();
    //fs::copy(release_dir.join("model-proxy"), &bin_dir.join("model-proxy")).unwrap();

    println!("cargo:rerun-if-changed=bin/");
}

fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_cargo = read_dir(p)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == OsString::from("Cargo.lock"));
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}
