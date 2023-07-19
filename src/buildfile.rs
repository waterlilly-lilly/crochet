use std::collections::BTreeMap;
use semver::Version;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
pub struct BuildFile {
    config: BuildFileConfigSection,
    dependencies: BTreeMap<String, String>,
    target: BuildFileTargetSection
}
#[derive(Deserialize, Debug)]
pub struct BuildFileConfigSection {
    id: String,
    version: String,
    name: String,
    description: String,
    entrypoint: String
}
#[derive(Deserialize, Debug)]
pub struct BuildFileTargetSection {
    datagen: String,
    codegen: String,
    jar: String
}