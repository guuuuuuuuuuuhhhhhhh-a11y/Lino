#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager};
use anyhow::Result;
use ldm_core::{self as core, distro_registry, models::*, env_manager};
use std::path::PathBuf;

#[tauri::command]
async fn cmd_list_distros() -> Result<Vec<Distro>, String> {
    let idx = load_index().await.map_err(|e| e.to_string())?;
    Ok(idx.distributions)
}

#[tauri::command]
async fn cmd_versions(distro_id: String) -> Result<Vec<DistroVersion>, String> {
    let idx = load_index().await.map_err(|e| e.to_string())?;
    let d = distro_registry::get_distro(&idx, &distro_id).ok_or("distro not found").map_err(|e: &str| e.to_string())?;
    Ok(d.versions.clone())
}

#[derive(serde::Deserialize)]
struct InstallReq { distro_id: String, version: String, backend: String }

#[tauri::command]
async fn cmd_install(req: InstallReq) -> Result<(), String> {
    let idx = load_index().await.map_err(|e| e.to_string())?;
    let d = distro_registry::get_distro(&idx, &req.distro_id).ok_or("distro not found").map_err(|e: &str| e.to_string())?;
    let ver = distro_registry::get_version(d, &req.version).ok_or("version not found").map_err(|e: &str| e.to_string())?;
    let backend = match req.backend.as_str() { "wsl" => BackendKind::Wsl, _ => BackendKind::Docker };
    let artifact = core::install::pick_artifact(ver, backend).map_err(|e| e.to_string())?;

    let env = InstalledEnv { name: format!("{}-{}", d.id, ver.version), distro_id: d.id.clone(), version: ver.version.clone(), backend, install_dir: None };

    let spec = match artifact {
        Artifact::DockerImage { image } => core::backends::InstallSpec { docker_image: Some(image.clone()), rootfs_path: None },
        Artifact::Rootfs { .. } => {
            let tmpdir = std::env::temp_dir().join("ldm"); std::fs::create_dir_all(&tmpdir).ok();
            core::install::prepare_for_backend(artifact, &tmpdir, |_dl, _total| {}).await.map_err(|e| e.to_string())?
        }
    };

    env_manager::add(env.clone()).map_err(|e| e.to_string())?;
    match backend {
        BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.install(&env, spec).map_err(|e| e.to_string())?; }
        BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.install(&env, spec).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

#[tauri::command]
async fn cmd_open_settings() -> Result<(), String> { Ok(()) }

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![cmd_list_distros, cmd_versions, cmd_install, cmd_open_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn load_index() -> anyhow::Result<DistributionsIndex> {
    let path = std::env::current_dir()?.join("distributions.json");
    ldm_core::distro_registry::load_from_file(&path).await
}