#![feature(result_option_inspect)]

extern crate core;

mod buildfile;
mod minecraft;
mod util;

use std::{env, fs};
use std::error::Error;
use std::fs::File;
use std::io::ErrorKind::AlreadyExists;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use log::{trace, debug, info, warn, error};
use reqwest::Client;
use tokio::runtime::Runtime;
use crate::buildfile::{BuildFile, load_build_file};
use crate::minecraft::get_merged_minecraft_jar;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    path: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool
}
#[derive(Subcommand, PartialEq)]
pub enum Commands {
    DryRun,
    Compile,
    Init
}
fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();

    let args =  Args::parse();
    let client = Client::new();
    let runtime = Runtime::new()?;

    env::set_current_dir(args.path.clone().unwrap_or(env::current_dir()?))?;
    trace!("cwd: {:?}", env::current_dir()?);
    match args.command {
        Commands::Compile | Commands::DryRun => {
            if args.command == Commands::DryRun {
                info!("Dry running compiler (no filesystem changes will be made)");
            } else {
                if fs::read_dir("out").is_err() {
                    debug!("Creating out directory");
                    fs::create_dir("out")?;
                    fs::create_dir("out/.cache")?;
                }

            }
            if args.debug {
                debug!("Debug mode enabled! This will print compiled data as it is generated")
            }
            trace!("stage 1: build file resolution");
            let build_file = load_build_file(&args)?;
            trace!("stage 2: downloading minecraft.jar and dependencies");
            let minecraft_jar = get_merged_minecraft_jar(&args, &client, &runtime, &build_file)?;

        }
        Commands::Init => {
            panic!("Not yet implemented")
        }
    }
    Ok(())
}
