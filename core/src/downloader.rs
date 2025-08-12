use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, RANGE};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt as _;

#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub downloaded: u64,
    pub total: Option<u64>,
}

pub async fn download_with_resume(url: &str, dest: &Path, progress: impl Fn(ProgressInfo) + Send + Sync + 'static) -> Result<()> {
    let client = reqwest::Client::new();

    let mut downloaded: u64 = 0;
    let total = content_length(&client, url).await.ok();

    if dest.exists() {
        downloaded = tokio::fs::metadata(dest).await?.len();
    }

    let mut headers = HeaderMap::new();
    if downloaded > 0 {
        headers.insert(RANGE, format!("bytes={}-", downloaded).parse().unwrap());
    }

    let resp = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .with_context(|| format!("request failed: {url}"))?;

    let status = resp.status();
    if !(status.is_success() || status.as_u16() == 206) {
        return Err(anyhow::anyhow!("http status {}", status));
    }

    let mut file = if downloaded > 0 { tokio::fs::OpenOptions::new().append(true).open(dest).await? } else { tokio::fs::File::create(dest).await? };

    let mut stream = resp.bytes_stream();
    let mut current = downloaded;
    while let Some(chunk) = stream.next().await.transpose()? {
        file.write_all(&chunk).await?;
        current += chunk.len() as u64;
        progress(ProgressInfo { downloaded: current, total });
    }

    Ok(())
}

async fn content_length(client: &reqwest::Client, url: &str) -> Result<u64> {
    let resp = client.head(url).send().await?;
    let len = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    Ok(len)
}