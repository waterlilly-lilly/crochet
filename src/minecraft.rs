use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use log::{info, trace};
use reqwest::Client;
use semver::Version;
use crate::Args;
use crate::buildfile::BuildFile;
use serde::Deserialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use crate::util::{download, download_file, download_json};

#[derive(Deserialize)]
pub struct MinecraftVersionManifestV2 {
    pub latest: HashMap<String, String>,
    versions: Vec<MinecraftVersionManifestV2Entry>
}
#[derive(Deserialize, Debug)]
pub struct MinecraftVersionManifestV2Entry {
    pub id: String,
    #[serde(alias = "type")]
    pub rtype: String,
    pub url: String,
    pub time: String,
    #[serde(alias = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(alias = "complianceLevel")]
    pub compliance_level: usize

}
pub struct MinecraftJar {
    pub json: MinecraftVersionJson,
    pub jar: PathBuf
}
#[derive(Deserialize, Debug)]
pub struct MinecraftVersionJson {
    pub arguments: Value,
    #[serde(alias = "assetIndex")]
    pub asset_index: AssetIndex,
    #[serde(alias = "complianceLevel")]
    pub compliance_level: usize,
    pub downloads: HashMap<String, MinecraftVersionJsonArtifact>,
    pub id: String,
    // pub libraries: Vec<MinecraftVersionJsonLibrary>
}
#[derive(Deserialize, Debug)]
pub struct MinecraftVersionJsonArtifact {
    pub path: Option<String>,
    pub sha1: String,
    pub size: usize,
    pub url: String,
}
#[derive(Deserialize, Debug)]
pub struct MinecraftVersionJsonLibrary {
    pub downloads: Value,
    pub name: String,
    pub rules: Value
}
#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: usize,
    #[serde(alias = "totalSize")]
    pub total_size: usize,
    pub url: String
}
/// Downloads the minecraft server and client jars, then merges them. Returns a merged jar.
pub fn get_merged_minecraft_jar(args: &Args, client: &Client, runtime: &Runtime, build_file: &BuildFile) -> Result<MinecraftJar, Box<dyn Error>>{
    let version = build_file.versions.minecraft.to_owned();
    let manifest = get_manifest(version.to_owned(), client, runtime)?;
    if args.debug {
        dbg!(&manifest);
    }
    info!("Downloading minecraft jars for version {}", &manifest.id);
    let version_json: MinecraftVersionJson = download_json(client, runtime, manifest.url.as_str())?;
    if args.debug {
        dbg!(&version_json.downloads);
    }
    let server_jar = runtime.block_on(download_file(client, &version_json.downloads["server"].url.as_str(), PathBuf::from("out/.cache")))?;
    let client_jar = runtime.block_on(download_file(client, &version_json.downloads["client"].url.as_str(), PathBuf::from("out/.cache")))?;

    Ok(MinecraftJar {
        json: version_json,
        jar: PathBuf::new()
    })
}
fn get_manifest(version: String, client: &Client, runtime: &Runtime) -> Result<MinecraftVersionManifestV2Entry, Box<dyn Error>> {
    let manifest: MinecraftVersionManifestV2 = download_json(client, runtime, "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")?;
    for v in manifest.versions.into_iter() {
        if v.id == version {
            return Ok(v);
        }
    }
    Err(Box::try_from("Requested version unable to be found!")?)
}