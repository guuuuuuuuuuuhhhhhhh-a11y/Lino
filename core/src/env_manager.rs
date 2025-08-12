use crate::models::{InstalledEnv, BackendKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, fs};

#[derive(Debug, Default, Serialize, Deserialize)]
struct State { envs: Vec<InstalledEnv> }

fn state_path() -> PathBuf {
    let dir = dirs::data_dir().unwrap_or(std::env::temp_dir()).join("ldm");
    std::fs::create_dir_all(&dir).ok();
    dir.join("state.json")
}

pub fn list() -> Result<Vec<InstalledEnv>> {
    let p = state_path();
    if !p.exists() { return Ok(vec![]); }
    let data = fs::read_to_string(p)?;
    let state: State = serde_json::from_str(&data)?;
    Ok(state.envs)
}

pub fn save(envs: &[InstalledEnv]) -> Result<()> {
    let p = state_path();
    let state = State { envs: envs.to_vec() };
    let data = serde_json::to_string_pretty(&state)?;
    fs::write(p, data)?;
    Ok(())
}

pub fn add(env: InstalledEnv) -> Result<()> {
    let mut envs = list()?;
    envs.retain(|e| e.name != env.name);
    envs.push(env);
    save(&envs)
}

pub fn remove(name: &str) -> Result<()> {
    let mut envs = list()?;
    envs.retain(|e| e.name != name);
    save(&envs)
}