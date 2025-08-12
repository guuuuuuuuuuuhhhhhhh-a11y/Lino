use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn verify_sha256(file: &Path, expected_hex: &str) -> Result<()> {
    let mut f = std::fs::File::open(file)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut f, &mut hasher)?;
    let got = hex::encode(hasher.finalize());
    if got.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(anyhow!("sha256 mismatch: got {got}, expected {expected_hex}"))
    }
}

pub async fn fetch_sha256_from_sumfile(url: &str, target_filename: &str) -> Result<String> {
    let text = reqwest::get(url).await?.text().await?;
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let (hash, file) = (parts[0], parts[1]);
            if file.ends_with(target_filename) || file == target_filename {
                return Ok(hash.to_string());
            }
        }
    }
    Err(anyhow!("hash for {target_filename} not found in sums file"))
}