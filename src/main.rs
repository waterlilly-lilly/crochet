mod buildfile;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use log::{trace, debug, info, warn, error};
use crate::buildfile::BuildFile;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, value_name = "FILE")]
    path: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool
}
#[derive(Subcommand)]
enum Commands {
    DryRun,
    Compile,
    Init
}
fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    let args =  Arc::new(Args::parse());
    match args.command {
        Commands::DryRun => {
            info!("Dry running compiler (no filesystem changes will be made)");
            if args.debug {
                debug!("Debug mode enabled! This will print compiled data as it is generated")
            }
            trace!("Locating build.toml file");
            let path = args.path.clone().unwrap_or(env::current_dir()?)
            .join("build.toml");
            trace!("trying build.toml at {:?}", path.clone());
            let mut build_file = File::open(path)?;
            let mut build_data = String::new();
            build_file.read_to_string(&mut build_data)?;
            let build: BuildFile = toml::from_str(build_data.as_str())?;
            if args.debug {
                dbg!(&build);
            }
        }
        Commands::Compile => {
            panic!("Not yet implemented")
        }
        Commands::Init => {
            panic!("Not yet implemented")
        }
    }
    Ok(())
}
