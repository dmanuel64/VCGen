use crate::config::CommandLineArgs;
use clap::Parser;

mod config;

fn main() {
    let args = CommandLineArgs::parse();
    println!("Hello, world!");
}
