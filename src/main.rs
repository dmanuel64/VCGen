use crate::config::CommandLineArgs;
use clap::Parser;
use std::error::Error;
use vcgen::{self, check_dependencies};
use vcgen::{commit_policy, VCGenerator};

mod config;

/// Gets the [`vcgen::WorkDivisionStrategy`] specified by the [`CommandLineArgs`].
fn strategy(args: &CommandLineArgs) -> vcgen::WorkDivisionStrategy {
    match args.strategy() {
        config::WorkDivisionStrategy::SUCCESSIVE => vcgen::WorkDivisionStrategy::SUCCESSIVE,
        config::WorkDivisionStrategy::PERCENTILE => vcgen::WorkDivisionStrategy::PERCENTILE,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse command line arguments
    let args = CommandLineArgs::parse();
    // check for any additional requirements
    check_dependencies(false)?;
    // create vulnerable code dataset with input arguments
    let df = VCGenerator::new(args.entries(), &args.dataset_file())
        .set_worker_threads(args.worker_threads())
        .set_max_repo_size(args.max_repo_size())
        .set_strategy(strategy(&args))
        .set_policy(commit_policy(&args.policy().to_string()))
        .set_quiet(false)
        .create_dataset()?;
    if cfg!(debug_assertions) {
        // Show sample DataFrame in debug mode
        println!("{}", df.head(None));
    }
    // Print generator results
    if df.shape().0 < args.entries() as usize {
        println!(
            "Note: Vulnerable Code Generator could not generate the desired amount of entries."
        );
        println!("Try loosening some of the options such as a lower vulnerable commit policy or increased repository size limit.")
    }
    println!(
        "Dataset saved with {} entries at {}",
        df.shape().0,
        args.dataset_file()
            .to_str()
            .expect("Could not get path to dataset.")
    );
    Ok(())
}
