use clap::{ArgGroup, Parser};
use std::path::PathBuf;

const DEFAULT_FLAWFINDER_PATH: &str = "/usr/bin/flawfinder";
const DEFAULT_CPPCHECK_PATH: &str = "/usr/bin/cppcheck";
const DEFAULT_INFER_PATH: &str = "/usr/bin/infer";

#[derive(Parser)]
#[clap(name = "Vulnerable Code Dataset Generator")]
#[clap(author = "Dylan Manuel", version = "1.0")]
#[clap(about = "Does awesome things", long_about = None)]
#[clap(group(
    ArgGroup::new("flawfinder")
        .required(false)
        .args(&["flawfinder-path", "disable-flawfinder"]),
))]
#[clap(group(
    ArgGroup::new("cppcheck")
        .required(false)
        .args(&["cppcheck-path", "disable-cppcheck"]),
))]
#[clap(group(
    ArgGroup::new("infer")
        .required(false)
        .args(&["infer-path", "disable-infer"]),
))]
pub struct CommandLineArgs {
    /// Number of dataset entries
    #[clap(parse(try_from_str = positive_value))]
    entries: i32,
    /// Path to save the dataset at
    dataset_file: PathBuf,
    /// Ratio of vulnerable code entries to benign code entries
    #[clap(short, long, parse(try_from_str = positive_percentage), value_name = "VULNERABILITY", default_value_t = 0.5)]
    ratio: f32,
    /// Path to Flawfinder
    #[clap(long, value_name = "PATH", default_value = DEFAULT_FLAWFINDER_PATH)]
    flawfinder_path: String,
    /// Will not use Flawfinder in creating the dataset
    #[clap(long)]
    disable_flawfinder: bool,
    /// Path to Cppcheck
    #[clap(long, value_name = "PATH", default_value = DEFAULT_CPPCHECK_PATH)]
    cppcheck_path: String,
    /// Will not use Cppcheck in creating the dataset
    #[clap(long)]
    disable_cppcheck: bool,
    /// Path to Infer
    #[clap(long, value_name = "PATH", default_value = DEFAULT_INFER_PATH)]
    infer_path: String,
    /// Will not use Infer in creating the dataset
    #[clap(long)]
    disable_infer: bool,
    #[clap(short, long, value_name = "AMOUNT", parse(try_from_str = positive_value), default_value_t = 4)]
    worker_threads: i32,
    #[clap(long, value_name = "KB")]
    max_repo_size: Option<u32>,
}

impl CommandLineArgs {
    pub fn entries(&self) -> i32 {
        self.entries
    }

    pub fn dataset_file(&self) -> &PathBuf {
        &self.dataset_file
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn worker_threads(&self) -> i32 {
        self.worker_threads
    }

    pub fn max_repo_size(&self) -> Option<u32> {
        self.max_repo_size
    }
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
