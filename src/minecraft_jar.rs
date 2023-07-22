use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use log::{debug, info, trace};
use reqwest::Client;
use semver::Version;
use crate::Args;
use crate::buildfile::BuildFile;
use serde::Deserialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use crate::util::{download, download_file, download_json, file_matches_hash, read_json};

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
fn get_manifest(version: String, client: &Client, runtime: &Runtime) -> Result<MinecraftVersionManifestV2Entry, Box<dyn Error>> {
    let manifest: MinecraftVersionManifestV2 = download_json(client, runtime, "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")?;
    for v in manifest.versions.into_iter() {
        if v.id == version {
            return Ok(v);
        }
    }
    Err(Box::try_from("Requested version unable to be found!")?)
}
/// Downloads the minecraft client jar (which also contains the server jar).
pub fn download_jar(args: &Args, client: &Client, runtime: &Runtime, build_file: &BuildFile) -> Result<(MinecraftVersionJson, PathBuf), Box<dyn Error>> {
    let version = build_file.versions.minecraft.to_owned();

    let minecraft_json = PathBuf::from(format!("out/.cache/mcjar/{version}.json"));
    let minecraft_jar = PathBuf::from(format!("out/.cache/mcjar/minecraft-client-{version}.jar"));

    let result = if !(Path::exists(&minecraft_json) && Path::exists(&minecraft_jar)) {
        let manifest = get_manifest(version.to_owned(), client, runtime)?;

        let version_data = runtime.block_on(download_file(client, manifest.url.as_str(), &minecraft_json))?;
        let version_data = std::str::from_utf8(version_data.as_slice())?;
        let version_json: MinecraftVersionJson = serde_json::from_str(version_data)?;

        let client_jar_entry = &version_json.downloads["client"];

        runtime.block_on(download_file(client, client_jar_entry.url.as_str(), &minecraft_jar))?;

        (version_json, minecraft_jar)
    } else {
        debug!("Found version.json and minecraft-client.jar, skipping download...");
        (
            read_json::<MinecraftVersionJson, _>(&minecraft_json)?,
            minecraft_jar
        )
    };
    Ok(result)
}