use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionsIndex {
    pub updated_at: DateTime<Utc>,
    pub distributions: Vec<Distro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distro {
    pub id: String,
    pub name: String,
    pub vendor: Option<String>,
    pub website: Option<String>,
    pub backends: Vec<BackendKind>,
    pub versions: Vec<DistroVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Wsl,
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroVersion {
    pub version: String,
    pub channel: Option<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Artifact {
    Rootfs { download_url: String, sha256: Option<String>, sha256_url: Option<String> },
    DockerImage { image: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledEnv {
    pub name: String,
    pub distro_id: String,
    pub version: String,
    pub backend: BackendKind,
    pub install_dir: Option<String>,
}