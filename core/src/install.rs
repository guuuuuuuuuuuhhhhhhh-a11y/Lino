use crate::models::{Artifact, BackendKind, Distro, DistroVersion};
use crate::backends::InstallSpec;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

pub fn find_version<'a>(distro: &'a Distro, version: &str) -> Result<&'a DistroVersion> {
    distro.versions.iter().find(|v| v.version == version).ok_or_else(|| anyhow!("version not found"))
}

pub fn pick_artifact<'a>(ver: &'a DistroVersion, backend: BackendKind) -> Result<&'a Artifact> {
    match backend {
        BackendKind::Docker => ver.artifacts.iter().find(|a| matches!(a, Artifact::DockerImage { .. })).ok_or_else(|| anyhow!("no docker image artifact")),
        BackendKind::Wsl => ver.artifacts.iter().find(|a| matches!(a, Artifact::Rootfs { .. })).ok_or_else(|| anyhow!("no rootfs artifact")),
    }
}

pub async fn prepare_for_backend(artifact: &Artifact, download_dir: &Path, progress: impl Fn(u64, Option<u64>) + Send + Sync + 'static) -> Result<InstallSpec> {
    match artifact {
        Artifact::DockerImage { image } => Ok(InstallSpec { docker_image: Some(image.clone()), rootfs_path: None }),
        Artifact::Rootfs { download_url, sha256, sha256_url } => {
            let file_name = download_url.split('/').last().unwrap_or("rootfs.tar.gz");
            let dest = download_dir.join(file_name);
            crate::downloader::download_with_resume(download_url, &dest, move |p| progress(p.downloaded, p.total)).await?;
            if let Some(hash) = sha256 {
                crate::hash::verify_sha256(&dest, hash)?;
            } else if let Some(list_url) = sha256_url {
                let expected = crate::hash::fetch_sha256_from_sumfile(list_url, file_name).await?;
                crate::hash::verify_sha256(&dest, &expected)?;
            }
            // If gzip, optionally decompress to .tar because wsl --import prefers .tar
            let rootfs_path = if dest.extension().and_then(|e| e.to_str()) == Some("gz") {
                let tar_path = dest.with_extension(""); // strip .gz -> .tar
                gunzip_to(&dest, &tar_path)?;
                tar_path
            } else {
                dest
            };
            Ok(InstallSpec { docker_image: None, rootfs_path: Some(rootfs_path) })
        }
    }
}

fn gunzip_to(src_gz: &Path, dest_tar: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    let mut input = std::fs::File::open(src_gz)?;
    let mut decoder = GzDecoder::new(&mut input);
    let mut out = std::fs::File::create(dest_tar)?;
    std::io::copy(&mut decoder, &mut out).context("decompressing gz")?;
    Ok(())
}