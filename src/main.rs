use crate::config::CommandLineArgs;
use clap::Parser;
use vcgen;
use vcgen::create_dataset;

mod config;

fn main() {
    let args = CommandLineArgs::parse();
    let _results = create_dataset(
        args.entries(),
        &args.dataset_file(),
        args.ratio(),
        args.worker_threads(),
    );
    println!("Hello, world!");
}
