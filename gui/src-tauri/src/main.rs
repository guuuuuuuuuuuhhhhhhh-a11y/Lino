#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, AppHandle};
use anyhow::Result;
use ldm_core::{self as core, distro_registry, models::*, env_manager};
use serde::Serialize;

static DISTRO_JSON: &str = include_str!("assets/distributions.json");

#[derive(Serialize, Clone)]
struct InstallProgressPayload {
    distro_id: String,
    version: String,
    phase: String,
    downloaded: u64,
    total: Option<u64>,
}

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
async fn cmd_install(app: AppHandle, req: InstallReq) -> Result<(), String> {
    let idx = load_index().await.map_err(|e| e.to_string())?;
    let d = distro_registry::get_distro(&idx, &req.distro_id).ok_or("distro not found").map_err(|e: &str| e.to_string())?;
    let ver = distro_registry::get_version(d, &req.version).ok_or("version not found").map_err(|e: &str| e.to_string())?;
    let backend = match req.backend.as_str() { "wsl" => BackendKind::Wsl, _ => BackendKind::Docker };
    let artifact = core::install::pick_artifact(ver, backend).map_err(|e| e.to_string())?;

    let env = InstalledEnv { name: format!("{}-{}", d.id, ver.version), distro_id: d.id.clone(), version: ver.version.clone(), backend, install_dir: None };

    let distro_id = d.id.clone();
    let ver_s = ver.version.clone();

    let spec = match artifact {
        Artifact::DockerImage { image } => core::backends::InstallSpec { docker_image: Some(image.clone()), rootfs_path: None },
        Artifact::Rootfs { .. } => {
            let tmpdir = std::env::temp_dir().join("ldm"); std::fs::create_dir_all(&tmpdir).ok();
            app.emit("install-progress", InstallProgressPayload { distro_id: distro_id.clone(), version: ver_s.clone(), phase: "download".into(), downloaded: 0, total: None }).ok();
            core::install::prepare_for_backend(artifact, &tmpdir, |dl, total| {
                let _ = app.emit("install-progress", InstallProgressPayload { distro_id: distro_id.clone(), version: ver_s.clone(), phase: "download".into(), downloaded: dl, total });
            }).await.map_err(|e| e.to_string())?
        }
    };

    env_manager::add(env.clone()).map_err(|e| e.to_string())?;
    app.emit("install-progress", InstallProgressPayload { distro_id: distro_id.clone(), version: ver_s.clone(), phase: "import".into(), downloaded: 0, total: None }).ok();
    match backend {
        BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.install(&env, spec).map_err(|e| e.to_string())?; }
        BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.install(&env, spec).map_err(|e| e.to_string())?; }
    }
    app.emit("install-progress", InstallProgressPayload { distro_id, version: ver_s, phase: "done".into(), downloaded: 0, total: None }).ok();
    Ok(())
}

#[tauri::command]
async fn cmd_open_settings() -> Result<(), String> { Ok(()) }

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cmd_list_distros, cmd_versions, cmd_install, cmd_open_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn load_index() -> anyhow::Result<DistributionsIndex> {
    let index: DistributionsIndex = serde_json::from_str(DISTRO_JSON)?;
    Ok(index)
}