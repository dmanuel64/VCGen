use crate::config::CommandLineArgs;
use clap::Parser;
use std::error::Error;
use vcgen::{self, check_dependencies};
use vcgen::VCGenerator;

mod config;

fn main() -> Result<(), Box<dyn Error>> {
    // parse command line arguments
    let args = CommandLineArgs::parse();
    // check for any additional requirements
    check_dependencies(args.disable_flawfinder())?;
    // create vulnerable code dataset with input arguments
    let df = VCGenerator::new(args.entries(), &args.dataset_file())
        .set_worker_threads(args.worker_threads())
        .set_vulnerability_ratio(args.ratio())
        .set_max_repo_size(args.max_repo_size())
        .set_quiet(false)
        .create_dataset()?;
    println!("{}\nProduced a {:?} DataFrame", df.head(None), df.shape());
    Ok(())
}
