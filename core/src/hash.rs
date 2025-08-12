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