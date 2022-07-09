use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "Vulnerable Code Dataset Generator")]
#[clap(author = "Dylan Manuel", version = "1.0")]
#[clap(about = "Does awesome things", long_about = None)]
pub struct CommandLineArgs {
    /// Number of desired dataset entries
    #[clap(parse(try_from_str = positive_value))]
    entries: i32,
    /// Path to save the dataset at
    dataset_file: PathBuf,
    /// Ratio of vulnerable code entries to benign code entries
    #[clap(short, long, parse(try_from_str = positive_percentage), value_name = "VULNERABILITY", default_value_t = 0.5)]
    ratio: f32,
    /// Excludes using Flawfinder as a source code static analyzer
    #[clap(long)]
    disable_flawfinder: bool,
    /// Excludes using Cppcheck as a source code static analyzer
    #[clap(long)]
    disable_cppcheck: bool,
    /// Excludes using Infer as a source code static analyzer
    #[clap(long)]
    disable_infer: bool,
    /// The amount of worker threads scanning for vulnerable code. Each worker
    /// thread works on one repository at a time, with the work equally divided
    #[clap(short, long, value_name = "AMOUNT", parse(try_from_str = positive_value), default_value_t = 4)]
    worker_threads: i32,
    /// An optional size limit in kilobytes on allowing worker threads to only
    /// clone trending repositories under the limit
    #[clap(short, long, value_name = "KB")]
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

    pub fn disable_flawfinder(&self) -> bool {
        self.disable_flawfinder
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
