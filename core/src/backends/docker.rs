use anyhow::{anyhow, Result};
use std::process::Command;
use crate::models::{BackendKind, InstalledEnv};
use super::{Backend, InstallSpec};

pub struct DockerBackend;

impl Backend for DockerBackend {
    fn kind(&self) -> BackendKind { BackendKind::Docker }

    fn list(&self) -> Result<Vec<InstalledEnv>> {
        Ok(vec![])
    }

    fn install(&self, env: &InstalledEnv, spec: InstallSpec) -> Result<()> {
        let image = spec.docker_image.ok_or_else(|| anyhow!("docker image not provided"))?;
        let status = Command::new("docker").args(["pull", &image]).status()?;
        if !status.success() { return Err(anyhow!("docker pull failed")); }
        let status = Command::new("docker").args(["run", "-d", "--name", &env.name, &image]).status()?;
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