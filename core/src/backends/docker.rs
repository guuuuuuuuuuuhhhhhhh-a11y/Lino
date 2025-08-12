use anyhow::{anyhow, Result};
use std::process::Command;
use crate::models::{BackendKind, InstalledEnv};
use super::Backend;

pub struct DockerBackend;

impl Backend for DockerBackend {
    fn kind(&self) -> BackendKind { BackendKind::Docker }

    fn list(&self) -> Result<Vec<InstalledEnv>> {
        // Minimal stub: list running containers named by our convention could be implemented later
        Ok(vec![])
    }

    fn install(&self, env: &InstalledEnv, _artifact_path: &std::path::Path) -> Result<()> {
        let image = format!("{}:{}", env.distro_id, env.version); // placeholder; normally from artifact
        let status = Command::new("docker")
            .args(["pull", &image])
            .status()?;
        if !status.success() { return Err(anyhow!("docker pull failed")); }
        let status = Command::new("docker")
            .args(["run", "-d", "--name", &env.name, &image])
            .status()?;
        if !status.success() { return Err(anyhow!("docker run failed")); }
        Ok(())
    }

    fn start(&self, name: &str) -> Result<()> {
        let status = Command::new("docker").args(["start", name]).status()?;
        anyhow::ensure!(status.success(), "docker start failed");
        Ok(())
    }

    fn stop(&self, name: &str) -> Result<()> {
        let status = Command::new("docker").args(["stop", name]).status()?;
        anyhow::ensure!(status.success(), "docker stop failed");
        Ok(())
    }

    fn remove(&self, name: &str) -> Result<()> {
        let status = Command::new("docker").args(["rm", "-f", name]).status()?;
        anyhow::ensure!(status.success(), "docker rm failed");
        Ok(())
    }
}