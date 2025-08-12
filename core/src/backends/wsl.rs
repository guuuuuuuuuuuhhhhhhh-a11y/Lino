use anyhow::{anyhow, Result};
use crate::models::{BackendKind, InstalledEnv};
use super::Backend;

pub struct WslBackend;

impl Backend for WslBackend {
    fn kind(&self) -> BackendKind { BackendKind::Wsl }

    fn list(&self) -> Result<Vec<InstalledEnv>> {
        #[cfg(windows)]
        {
            let out = std::process::Command::new("wsl.exe").args(["-l", "-v"]).output()?;
            anyhow::ensure!(out.status.success(), "wsl -l -v failed");
            let _text = String::from_utf8_lossy(&out.stdout);
            // TODO: parse output into InstalledEnv entries with best effort
        }
        Ok(vec![])
    }

    fn install(&self, env: &InstalledEnv, artifact_path: &std::path::Path) -> Result<()> {
        #[cfg(windows)]
        {
            let install_dir = env.install_dir.clone().unwrap_or_else(|| format!("C:\\WSL\\{}", env.name));
            let status = std::process::Command::new("wsl.exe")
                .args(["--import", &env.name, &install_dir, &artifact_path.to_string_lossy(), "--version", "2"]) 
                .status()?;
            anyhow::ensure!(status.success(), "WSL import failed");
        }
        #[cfg(not(windows))]
        {
            let _ = (env, artifact_path);
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