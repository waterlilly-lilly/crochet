use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use log::trace;
use semver::Version;
use serde::{Serialize, Deserialize};
use crate::Args;

#[derive(Deserialize, Debug)]
pub struct BuildFile {
    pub config: BuildFileConfigSection,
    pub repositories: Option<BTreeMap<String, String>>,
    pub dependencies: Option<BTreeMap<String, String>>,
    pub versions: BuildFileVersions,
    pub quilt: Option<BuildFileQuiltVersions>,
    pub target: BuildFileTargetSection
}
#[derive(Deserialize, Debug)]
pub struct BuildFileVersions {
    pub java: String,
    pub crochet: String,
    pub minecraft: String
}
#[derive(Deserialize, Debug)]
pub struct BuildFileQuiltVersions {
    pub mappings: String,
    pub loader: String,
    pub api: String
}
#[derive(Deserialize, Debug)]
pub struct BuildFileConfigSection {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub entrypoint: String
}
#[derive(Deserialize, Debug)]
pub struct BuildFileTargetSection {
    pub datagen: String,
    pub codegen: String,
    pub jar: String
}
pub fn load_build_file(args: &Args) -> Result<BuildFile, Box<dyn Error>> {
    let path = env::current_dir()?.join("build.toml");

    let mut build_file = File::open(path)?;
    let mut build_data = String::new();

    build_file.read_to_string(&mut build_data)?;

    let build: BuildFile = toml::from_str(build_data.as_str())?;

    if args.debug {
        dbg!(&build);
    }

    Ok(build)
}