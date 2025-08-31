use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use common::dir::AppDir;
use env_logger::WriteStyle;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

pub use log;
/// 初始化日志
pub fn init_log() {
    if cfg!(debug_assertions) {
        init_log_for_debug();
    } else {
        init_log_for_release();
    }
}

// 创建一个自定义的写入器，可以同时写入多个目标
struct MultiWriter {
    writers: Vec<Box<dyn Write + Send>>,
}

impl MultiWriter {
    fn new() -> Self {
        Self {
            writers: Vec::new(),
        }
    }

    fn add_writer<W: Write + Send + 'static>(&mut self, writer: W) {
        self.writers.push(Box::new(writer));
    }
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut result = Ok(buf.len());
        for writer in &mut self.writers {
            match writer.write_all(buf) {
                Ok(_) => {}
                Err(e) => result = Err(e),
            }
        }
        result.map(|_| buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}

fn init_log_for_release() {
    fs::create_dir_all(AppDir::log_dir()).expect("Failed to create log directory");
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(AppDir::log_dir().join("app.log"))
        .unwrap();
    // 创建多目标写入器
    let mut multi_writer = MultiWriter::new();
    multi_writer.add_writer(std::io::stderr());
    multi_writer.add_writer(log_file);

    env_logger::Builder::new()
        .filter_module("rocket", log::LevelFilter::Warn)
        .filter_module("rocket::response::debug", log::LevelFilter::Error)
        .filter_module("lance::dataset", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format(|buf, record| {
            let level = record.level().as_str();
            writeln!(
                buf,
                "[{}][{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                level,
                record.args()
            )
        })
        .write_style(WriteStyle::Always)
        .target(env_logger::Target::Pipe(Box::new(multi_writer)))
        .init();
}
fn init_log_for_debug() {
    env_logger::Builder::new()
        .filter_module("rocket::server", log::LevelFilter::Warn)
        //.filter_module("rbatis", log::LevelFilter::Debug)
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .format(|buf, record| {
            let module_path = record.module_path().unwrap_or("<unknown>");
            let file_path = record.file().unwrap_or("<unknown>");
            let module_path_truncated = if module_path.len() > 20 {
                &module_path[module_path.len().saturating_sub(20)..]
            } else {
                module_path
            };
            let file_path_truncated = if file_path.len() > 20 {
                &file_path[file_path.len().saturating_sub(20)..]
            } else {
                file_path
            };
            let level = match record.level() {
                log::Level::Error => Red.bold().paint("ERROR"),
                log::Level::Warn => Yellow.bold().paint("WARN"),
                log::Level::Info => Green.bold().paint("INFO"),
                log::Level::Debug => Blue.bold().paint("DEBUG"),
                log::Level::Trace => Cyan.bold().paint("TRACE"),
            };
            writeln!(
                buf,
                "[{}][{}][{:>20}][{:>20}:{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                level,
                module_path_truncated,
                file_path_truncated,
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .write_style(WriteStyle::Always)
        .init();
}
