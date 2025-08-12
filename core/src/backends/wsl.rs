use anyhow::{anyhow, Result};
use crate::models::{BackendKind, InstalledEnv};
use super::{Backend, InstallSpec};

pub struct WslBackend;

impl Backend for WslBackend {
    fn kind(&self) -> BackendKind { BackendKind::Wsl }

    fn list(&self) -> Result<Vec<InstalledEnv>> {
        #[cfg(windows)]
        {
            let out = std::process::Command::new("wsl.exe").args(["-l", "-v"]).output()?;
            anyhow::ensure!(out.status.success(), "wsl -l -v failed");
        }
        Ok(vec![])
    }

    fn install(&self, env: &InstalledEnv, spec: InstallSpec) -> Result<()> {
        #[cfg(windows)]
        {
            let rootfs = spec.rootfs_path.ok_or_else(|| anyhow!("rootfs path not provided"))?;
            let install_dir = env.install_dir.clone().unwrap_or_else(|| format!("C:\\WSL\\{}", env.name));
            let status = std::process::Command::new("wsl.exe")
                .args(["--import", &env.name, &install_dir, &rootfs.to_string_lossy(), "--version", "2"]) 
                .status()?;
            anyhow::ensure!(status.success(), "WSL import failed");
        }
        #[cfg(not(windows))]
        {
            let _ = (env, spec);
            return Err(anyhow!("WSL backend is only available on Windows"));
        }
        Ok(())
    }

    fn start(&self, name: &str) -> Result<()> {
        #[cfg(windows)]
        {
            let status = std::process::Command::new("wsl.exe").args(["-d", name]).status()?;
            anyhow::ensure!(status.success(), "wsl start failed");
            return Ok(());
        }
        #[cfg(not(windows))]
        { return Err(anyhow!("WSL backend is only available on Windows")); }
    }

    fn stop(&self, _name: &str) -> Result<()> { Ok(()) }

    fn remove(&self, name: &str) -> Result<()> {
        #[cfg(windows)]
        {
            let status = std::process::Command::new("wsl.exe").args(["--unregister", name]).status()?;
            anyhow::ensure!(status.success(), "wsl unregister failed");
            return Ok(());
        }
        #[cfg(not(windows))]
        { return Err(anyhow!("WSL backend is only available on Windows")); }
    }
}