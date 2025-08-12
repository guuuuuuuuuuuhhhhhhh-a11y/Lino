use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ldm_core::{self as core, distro_registry, models::*, env_manager};
use ldm_core::backends::{Backend, InstallSpec};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use indicatif::{ProgressBar, ProgressStyle};

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
    Snapshot { name: String, #[arg(long)] output: PathBuf },
    Restore { name: String, #[arg(long)] input: PathBuf, #[arg(long)] backend: BackendOpt, #[arg(long)] location: Option<PathBuf> },
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
            let idx = load_index(cli.registry).await?;
            let d = distro_registry::get_distro(&idx, &distro_id).expect("distro not found");
            let ver = distro_registry::get_version(d, &version).expect("version not found");
            let backend_kind = match backend { BackendOpt::Wsl => BackendKind::Wsl, BackendOpt::Docker => BackendKind::Docker };
            let artifact = core::install::pick_artifact(ver, backend_kind.clone()).expect("artifact not found");

            let env = InstalledEnv { name: name.clone(), distro_id: d.id.clone(), version: version.clone(), backend: backend_kind.clone(), install_dir: location.map(|p| p.to_string_lossy().to_string()) };

            let spec = match artifact {
                Artifact::DockerImage { image } => InstallSpec { docker_image: Some(image.clone()), rootfs_path: None },
                Artifact::Rootfs { .. } => {
                    let pb = ProgressBar::new(0);
                    pb.set_style(ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})").unwrap());
                    pb.set_message("Downloading rootfs:");
                    let tmpdir = std::env::temp_dir().join("ldm"); std::fs::create_dir_all(&tmpdir).ok();
                    let pb2 = pb.clone();
                    let spec = core::install::prepare_for_backend(artifact, &tmpdir, move |dl, total| {
                        if let Some(t) = total { pb2.set_length(t); }
                        pb2.set_position(dl);
                    }).await?;
                    pb.finish_with_message("Download complete");
                    spec
                }
            };

            env_manager::add(env.clone())?;
            match backend_kind {
                BackendKind::Docker => { let be = core::backends::docker::DockerBackend; be.install(&env, spec)?; }
                BackendKind::Wsl => { let be = core::backends::wsl::WslBackend; be.install(&env, spec)?; }
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
        Commands::Snapshot { name, output } => {
            let envs = env_manager::list()?;
            let env = envs.iter().find(|e| e.name == name).expect("env not found");
            match env.backend {
                BackendKind::Docker => {
                    // docker commit + save
                    let image = format!("ldm-snap-{}:latest", name);
                    std::process::Command::new("docker").args(["commit", &name, &image]).status()?;
                    std::process::Command::new("docker").args(["save", "-o"]).arg(&output).arg(&image).status()?;
                }
                BackendKind::Wsl => {
                    #[cfg(windows)] {
                        std::process::Command::new("wsl.exe").args(["--export", &name, &output.to_string_lossy()]).status()?;
                    }
                    #[cfg(not(windows))] { anyhow::bail!("WSL snapshot only on Windows"); }
                }
            }
            println!("snapshot saved to {}", output.display());
        }
        Commands::Restore { name, input, backend, location } => {
            let backend_kind = match backend { BackendOpt::Wsl => BackendKind::Wsl, BackendOpt::Docker => BackendKind::Docker };
            let env = InstalledEnv { name: name.clone(), distro_id: "restored".into(), version: "restored".into(), backend: backend_kind.clone(), install_dir: location.map(|p| p.to_string_lossy().to_string()) };
            match backend_kind {
                BackendKind::Docker => {
                    // docker load, then run
                    std::process::Command::new("docker").args(["load", "-i"]).arg(&input).status()?;
                    // user should provide image name; for simplicity assume archive contains one image tagged ldm-snap-<name>:latest
                    let image = format!("ldm-snap-{}:latest", name);
                    let be = core::backends::docker::DockerBackend; be.install(&env, InstallSpec { docker_image: Some(image), rootfs_path: None })?;
                }
                BackendKind::Wsl => {
                    #[cfg(windows)] {
                        let be = core::backends::wsl::WslBackend; be.install(&env, InstallSpec { docker_image: None, rootfs_path: Some(input.clone()) })?;
                    }
                    #[cfg(not(windows))] { anyhow::bail!("WSL restore only on Windows"); }
                }
            }
            env_manager::add(env)?;
            println!("restored {}", name);
        }
    }

    Ok(())
}

async fn load_index(registry: Option<PathBuf>) -> Result<DistributionsIndex> {
    let path = registry.unwrap_or_else(|| std::env::current_dir().unwrap().join("distributions.json"));
    ldm_core::distro_registry::load_from_file(&path).await
}