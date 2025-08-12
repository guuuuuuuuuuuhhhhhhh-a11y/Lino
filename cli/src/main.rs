use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ldm_core::{self as core, distro_registry, models::*, env_manager};
use ldm_core::backends::Backend;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "ldm", version, about = "Linux Distro Manager")]
struct Cli {
    #[arg(long, global = true)]
    registry: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Doctor,
    ListDistros,
    Versions { distro_id: String },
    Install { distro_id: String, #[arg(long)] version: String, #[arg(long)] backend: BackendOpt, #[arg(long)] name: String, #[arg(long)] location: Option<PathBuf>, #[arg(long)] verify: bool },
    List,
    Start { name: String },
    Stop { name: String },
    Remove { name: String },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BackendOpt { Wsl, Docker }

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
            ldm_core::health::check_internet("https://github.com").await.ok();
            #[cfg(windows)] { let _ = ldm_core::health::check_tool("wsl.exe"); }
            let _ = ldm_core::health::check_tool("docker");
            println!("OK");
        }
        Commands::ListDistros => {
            let idx = load_index(cli.registry).await?;
            for d in distro_registry::list_distros(&idx) { println!("{}\t{}", d.id, d.name); }
        }
        Commands::Versions { distro_id } => {
            let idx = load_index(cli.registry).await?;
            let d = distro_registry::get_distro(&idx, &distro_id).expect("distro not found");
            for v in distro_registry::get_versions(d) { println!("{}\t{}", v.version, v.channel.as_deref().unwrap_or("")); }
        }
        Commands::Install { distro_id, version, backend, name, location, verify: _ } => {
            // Minimal: record env and call backend install (image pull/WSL import to be refined per artifact)
            let env = InstalledEnv { name: name.clone(), distro_id, version, backend: match backend { BackendOpt::Wsl => BackendKind::Wsl, BackendOpt::Docker => BackendKind::Docker }, install_dir: location.map(|p| p.to_string_lossy().to_string()) };
            env_manager::add(env.clone())?;
            match backend {
                BackendOpt::Docker => {
                    let be = core::backends::docker::DockerBackend; be.install(&env, std::path::Path::new(""))?;
                }
                BackendOpt::Wsl => {
                    let be = core::backends::wsl::WslBackend; be.install(&env, std::path::Path::new("<rootfs.tar.gz>"))?; // placeholder
                }
            }
            println!("installed {}", name);
        }
        Commands::List => {
            for e in env_manager::list()? { println!("{}\t{:?}\t{} {}", e.name, e.backend, e.distro_id, e.version); }
        }
        Commands::Start { name } => {
            let envs = env_manager::list()?;
            let env = envs.iter().find(|e| e.name == name).expect("env not found").clone();
            match env.backend {
                BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.start(&name)?; }
                BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.start(&name)?; }
            }
        }
        Commands::Stop { name } => {
            let envs = env_manager::list()?;
            let env = envs.iter().find(|e| e.name == name).expect("env not found").clone();
            match env.backend {
                BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.stop(&name)?; }
                BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.stop(&name)?; }
            }
        }
        Commands::Remove { name } => {
            let envs = env_manager::list()?;
            if let Some(env) = envs.iter().find(|e| e.name == name) {
                match env.backend {
                    BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.remove(&name)?; }
                    BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.remove(&name)?; }
                }
            }
            ldm_core::env_manager::remove(&name)?;
            println!("removed {}", name);
        }
    }

    Ok(())
}

async fn load_index(registry: Option<PathBuf>) -> Result<DistributionsIndex> {
    let path = registry.unwrap_or_else(|| std::env::current_dir().unwrap().join("distributions.json"));
    ldm_core::distro_registry::load_from_file(&path).await
}