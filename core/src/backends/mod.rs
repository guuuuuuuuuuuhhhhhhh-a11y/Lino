pub mod wsl;
pub mod docker;

use anyhow::Result;
use crate::models::{BackendKind, InstalledEnv};

pub trait Backend {
    fn kind(&self) -> BackendKind;
    fn list(&self) -> Result<Vec<InstalledEnv>>;
    fn install(&self, env: &InstalledEnv, artifact_path: &std::path::Path) -> Result<()>;
    fn start(&self, name: &str) -> Result<()>;
    fn stop(&self, name: &str) -> Result<()>;
    fn remove(&self, name: &str) -> Result<()>;
}