use logging::log;
use std::env::temp_dir;
use std::path::PathBuf;
use std::process::{Child, Command};

pub(crate) struct EmbedApp {
    binary_path: PathBuf,
    child: Option<Child>,
}

pub struct BinaryName<'a>(pub &'a str);
impl std::fmt::Display for BinaryName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(windows) {
            write!(f, "{}.exe", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl AsRef<str> for BinaryName<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl EmbedApp {
    pub(crate) fn new(
        name: BinaryName,
        binary_data: &[u8],
        args: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = temp_dir().join(name.as_ref());
        std::fs::write(&path, binary_data)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }

        let child = Command::new(&path).args(args).spawn()?;

        log::info!("Process started with PID: {}", child.id());

        Ok(EmbedApp {
            child: Some(child),
            binary_path: path,
        })
    }

    fn kill(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for EmbedApp {
    fn drop(&mut self) {
        let _ = self.kill();
        let _ = std::fs::remove_file(&self.binary_path);
    }
}
