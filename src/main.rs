use crate::config::CommandLineArgs;
use clap::Parser;
use std::error::Error;
use vcgen::create_dataset;
use vcgen::{self, check_dependencies};

mod config;

fn main() -> Result<(), Box<dyn Error>> {
    // parse command line arguments
    let args = CommandLineArgs::parse();
    // check for any additional requirements
    check_dependencies(args.disable_flawfinder()).or_else(|err| {
        eprintln!("{}", err);
        Err(err)
    })?;
    let _results = create_dataset(
        args.entries(),
        &args.dataset_file(),
        args.ratio(),
        args.worker_threads(),
        args.max_repo_size(),
    );
    Ok(())
}
