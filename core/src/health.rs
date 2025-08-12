use anyhow::{Context, Result};
use sysinfo::Disks;
use which::which;
use std::path::Path;

pub async fn check_internet(url: &str) -> Result<()> {
    let status = reqwest::Client::new().head(url).send().await?.status();
    anyhow::ensure!(status.is_success(), "Internet not reachable: {}", status);
    Ok(())
}

pub fn check_disk_space(path: &Path, required_bytes: u64) -> Result<()> {
    let disks = Disks::new_with_refreshed_list();
    let path_str = path.to_string_lossy();
    let disk = disks.iter().find(|d| path_str.starts_with(d.mount_point().to_string_lossy().as_ref()));
    if let Some(disk) = disk {
        let available = disk.available_space();
        anyhow::ensure!(available >= required_bytes, "Insufficient disk space. Available: {} bytes, required: {} bytes", available, required_bytes);
    }
    Ok(())
}

pub fn check_tool(tool: &str) -> Result<()> {
    which(tool).with_context(|| format!("{} not found in PATH", tool))?;
    Ok(())
}