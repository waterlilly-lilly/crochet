use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::future;
use std::io::Write as _;
use std::fmt::Write;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{info, trace};
use reqwest::{Client, Response};
use serde::Deserialize;
use tokio::runtime::Runtime;

// substantial portions of this function taken from:
// https://gist.github.com/giuliano-oliveira/4d11d6b3bb003dba3a1b53f43d81b30d
pub async fn download(client: &Client, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        info!("Downloading {url}");
        let mut res = client
            .get(url)
            .send()
            .await?;
        let size = res
            .content_length().ok_or(format!("Failed to get content length"))?;
    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        let mut downloaded: u64 = 0;
        let mut out: Vec<u8> = Vec::new();

        while let Some(chunk) = res.chunk().await? {
            out.append(&mut chunk.to_vec());
            let new = min(downloaded + (chunk.len() as u64), size);
            downloaded = new;
            pb.set_position(new);
        }
    pb.finish_and_clear();
    Ok(out)
}
pub async fn download_file(client: &Client, url: &str, dir: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let path = PathBuf::from(url.rsplit_once("/").unwrap().1);
    let data = download(client, url).await?;
    let file_path = dir.join(path);
    dbg!(&file_path);
    let mut file = File::create(file_path.clone())?;
    file.write_all(data.as_slice())?;
    Ok(file_path)
}
pub fn download_json<T: for<'a> Deserialize<'a>>(client: &Client, runtime: &Runtime, url: &str) -> Result<T, Box<dyn Error>> {
    let bytes = runtime.block_on(download(client, url))?;
    let data = std::str::from_utf8(bytes.as_slice())?;
    let deserialized: T = serde_json::from_str(data)?;
    Ok(deserialized)
}
pub fn download_all(client: &Client, runtime: &Runtime, urls: Vec<String>, dir: &PathBuf) -> Result<usize, Box<dyn Error>> {
    runtime.block_on(async {
        let tasks: Vec<_> = urls
            .iter()
            .map(|url| {
                let url = url.clone();
                let dir = dir.clone();
                let client = client.clone();
                tokio::spawn(async move {
                    download_file(&client, url.as_str(), dir.clone()).await.expect("Failed to download");
                })
            })
            .collect();
        futures::future::join_all(tasks).await;
    });
    Ok(urls.len())
}