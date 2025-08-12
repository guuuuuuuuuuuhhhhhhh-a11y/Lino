pub mod wsl;
pub mod docker;

use anyhow::Result;
use crate::models::{BackendKind, InstalledEnv};

pub struct InstallSpec {
    pub docker_image: Option<String>,
    pub rootfs_path: Option<std::path::PathBuf>,
}

pub trait Backend {
    fn kind(&self) -> BackendKind;
    fn list(&self) -> Result<Vec<InstalledEnv>>;
    fn install(&self, env: &InstalledEnv, spec: InstallSpec) -> Result<()>;
    fn start(&self, name: &str) -> Result<()>;
    fn stop(&self, name: &str) -> Result<()>;
    fn remove(&self, name: &str) -> Result<()>;
}