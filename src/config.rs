use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(name = "Vulnerable Code Dataset Generator")]
#[clap(author = "Dylan Manuel", version = "1.0")]
#[clap(about = "Does awesome things", long_about = None)]
pub struct CommandLineArgs {
    /// Number of dataset entries
    #[clap(parse(try_from_str = positive_value))]
    entries: i32,
    /// Path to save the dataset at
    dataset_file: PathBuf,
    /// Ratio of vulnerable code entries to benign code entries
    #[clap(short, long, parse(try_from_str = positive_percentage), default_value_t = 0.5)]
    vulnerability_ratio: f32,
}

fn positive_value(arg: &str) -> Result<i32, String> {
    let parsed_value: Result<i32, _> = arg.parse();
    parsed_value
        .or_else(|_| Err(format!("{arg} must be an integer.")))
        .and_then(|value| {
            if value > 0 {
                Ok(value)
            } else {
                Err(format!("{value} must be a positive integer."))
            }
        })
}

fn positive_percentage(arg: &str) -> Result<f32, String> {
    let parsed_value: Result<f32, _> = arg.parse();
    parsed_value
        .or_else(|_| Err(format!("{arg} must be a decimal.")))
        .and_then(|value| {
            if value > 0.0 && value <= 1.0 {
                Ok(value)
            } else {
                Err(format!("{value} must be a percentage between 0% and 100%."))
            }
        })
}
