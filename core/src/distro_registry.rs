use crate::models::*;
use anyhow::Result;
use std::path::Path;

pub async fn load_from_file(path: &Path) -> Result<DistributionsIndex> {
    let data = tokio::fs::read_to_string(path).await?;
    let index: DistributionsIndex = serde_json::from_str(&data)?;
    Ok(index)
}

pub fn list_distros(index: &DistributionsIndex) -> Vec<&Distro> {
    index.distributions.iter().collect()
}

pub fn get_distro<'a>(index: &'a DistributionsIndex, id: &str) -> Option<&'a Distro> {
    index.distributions.iter().find(|d| d.id == id)
}

pub fn get_versions(distro: &Distro) -> Vec<&DistroVersion> {
    distro.versions.iter().collect()
}