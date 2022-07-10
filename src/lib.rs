use crate::dataset::vulnerabilities::save_dataset;
use dataset::vulnerabilities::create_dataset as generate_dataset;
use git::{
    search::{github_api_token, TrendingRepositories, GITHUB_API_VAR},
    vulnerability::{AnalyzedFile, VulnerableCommitIdentifier, VulnerableCommits},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use polars::prelude::DataFrame;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread::{self},
};
use tempfile::tempdir;
use utils::debug_print;
use vulnerability::tools::{Flawfinder, FLAWFINDER_ENV_VAR};
mod dataset;
mod git;
mod utils;
mod vulnerability;

/// Completes a closure with a progress spinner running
fn with_progress_spinner<F, R>(msg: &'static str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(msg);
    spinner.enable_steady_tick(125);
    let result = f();
    spinner.finish_using_style();
    result
}

/// Verifies that any other additional dependencies needed to generate vulnerable code is
/// resolved such as environment variables and argument combinations.
pub fn check_dependencies(skip_flawfinder: bool) -> Result<(), String> {
    if github_api_token().is_none() {
        Err(format!(
            "You must set the {} environment variable to your GitHub API personal access token.",
            GITHUB_API_VAR
        ))
    } else if !skip_flawfinder && Flawfinder::install_location().is_none() {
        Err(format!(
            "Flawfinder is not installed. \n{}\n{}\n{}",
            "Install Flawfinder with sudo apt-get install flawfinder,",
            "Download it from https://dwheeler.com/flawfinder/",
            format!(
                "Or set the {} environment variable of the path to the Flawfinder executable.",
                FLAWFINDER_ENV_VAR
            )
        ))
    } else if [skip_flawfinder].iter().all(|skipped| *skipped) {
        Err(format!("You must at least use one static analyzer."))
    } else {
        Ok(())
    }
}

/// Converts a string to its respective [`VulnerableCommitIdentifier`].
pub fn commit_policy(policy: &str) -> VulnerableCommitIdentifier {
    match policy.to_lowercase().as_str() {
        "strong" => VulnerableCommitIdentifier::STRONG,
        "medium" => VulnerableCommitIdentifier::MEDIUM,
        "low" => VulnerableCommitIdentifier::LOW,
        _ => panic!("Unknown policy: {}", policy),
    }
}

/// Indicates how work should be divided among worker threads
/// when cloning repositories and searching for vulnerabilities.
pub enum WorkDivisionStrategy {
    /// Each worker thread will grab the next successive repo from the top on the page
    SUCCESSIVE,
    /// Each worker thread will take a percentile of the top repos on the page
    PERCENTILE,
    /// Each worker thread will take a percentile on the page, but randomize the order
    RANDOM,
}

impl Default for WorkDivisionStrategy {
    fn default() -> Self {
        Self::SUCCESSIVE
    }
}

/// Builder that can generate datasets of vulnerable code from GitHub.com in
/// a JSON lines (`.jsonl`) format.
pub struct VCGenerator {
    entries: i32,
    dataset_path: PathBuf,
    vulnerability_ratio: f32,
    worker_threads: i32,
    strategy: WorkDivisionStrategy,
    policy: VulnerableCommitIdentifier,
    max_repo_size: Option<u32>,
    disable_flawfinder: bool,
    quiet: bool,
}

impl VCGenerator {
    /// Creates a new [`VCGenerator`] with default generator arguments, specifying
    /// the desired number of entries and where the dataset should be written to.
    pub fn new(entries: i32, dataset_path: &Path) -> Self {
        Self {
            entries: entries,
            dataset_path: PathBuf::from(dataset_path),
            vulnerability_ratio: 0.5,
            worker_threads: 4,
            strategy: WorkDivisionStrategy::default(),
            policy: VulnerableCommitIdentifier::default(),
            max_repo_size: None,
            disable_flawfinder: false,
            quiet: true,
        }
    }

    /// Sets the number of entries to generate.
    pub fn set_entries(&mut self, entries: i32) -> &mut Self {
        self.entries = entries;
        self
    }

    /// Sets the path to where the dataset should be written to.
    pub fn set_dataset_path(&mut self, dataset_path: &Path) -> &mut Self {
        self.dataset_path = PathBuf::from(dataset_path);
        self
    }

    /// Sets the ratio of how much vulnerable entries to benign entires should be
    /// collected for the dataset.
    pub fn set_vulnerability_ratio(&mut self, vulnerability_ratio: f32) -> &mut Self {
        self.vulnerability_ratio = vulnerability_ratio;
        self
    }

    /// Sets how many worker threads should contribute to cloning repositories and
    /// scanning for vulnerable code.
    pub fn set_worker_threads(&mut self, worker_threads: i32) -> &mut Self {
        self.worker_threads = worker_threads;
        self
    }

    /// Sets the optional size limit in kilobytes of how big repositories should be cloned and
    /// scanned.
    pub fn set_max_repo_size(&mut self, max_repo_size: Option<u32>) -> &mut Self {
        self.max_repo_size = max_repo_size;
        self
    }

    /// Sets if Flawfinder should be enabled/disabled when collecting entries.
    pub fn set_disable_flawfinder(&mut self, disable_flawfinder: bool) -> &mut Self {
        self.disable_flawfinder = disable_flawfinder;
        self
    }

    /// Sets if verbose progress bars and messages should be printed while the
    /// dataset is being generated.
    pub fn set_quiet(&mut self, quiet: bool) -> &mut Self {
        self.quiet = quiet;
        self
    }

    /// Gets the slice of repos the worker thread is supposed to work on based on the
    /// [`WorkDivisionStrategy`].
    fn worker_slice(&self, worker: i32, trending_repo_urls: Vec<String>) -> Vec<String> {
        let worker = worker as usize;
        let slice_size = trending_repo_urls.len() / self.worker_threads as usize;
        match self.strategy {
            WorkDivisionStrategy::SUCCESSIVE => trending_repo_urls
                [worker..trending_repo_urls.len()]
                .iter()
                .cloned()
                .step_by(self.worker_threads as usize)
                .collect(),
            WorkDivisionStrategy::PERCENTILE => {
                let start_idx = worker * slice_size;
                let end_idx = if worker == self.worker_threads as usize - 1 {
                    trending_repo_urls.len()
                } else {
                    start_idx + slice_size
                };
                trending_repo_urls[start_idx..end_idx].to_vec()
            }
            WorkDivisionStrategy::RANDOM => todo!("Random strategy"),
        }
    }

    /// Gets the amount of entries the worker thread must collect to generate
    /// the dataset
    fn worker_quota(&self, worker: i32) -> i32 {
        // last worker thread must get any remaining extra work
        if worker == self.worker_threads - 1 {
            self.entries - (self.entries / self.worker_threads * worker)
        } else {
            self.entries / self.worker_threads
        }
    }

    /// Creates the [`DataFrame`] of entries and saves it to the output file.
    fn generate_dataset(&self, vulnerabilities: Vec<AnalyzedFile>) -> DataFrame {
        let dataset_progress =
            if self.quiet {
                None
            } else {
                let pb = ProgressBar::new(vulnerabilities.len() as u64);
                pb.set_style(ProgressStyle::default_bar().template(
                    "[{elapsed_precise}] {bar:40.green/yellow} {pos:>7}/{len:7} {wide_msg}",
                ));
                Some(pb)
            };
        let mut df = generate_dataset(vulnerabilities, dataset_progress.as_ref());
        save_dataset(&mut df, &self.dataset_path);
        if let Some(pb) = dataset_progress {
            pb.finish_using_style();
        }
        df
    }

    /// Creates the dataset of vulnerable/benign entries with verbose progress output.
    fn create_dataset_verbose(&self) -> Result<DataFrame, String> {
        // get URLs of trending repositories from GitHub
        let trending_repos =
            with_progress_spinner("Collecting trending C repositories from GitHub...", || {
                TrendingRepositories::default()
            });
        let trending_repo_urls = Arc::new(trending_repos.repos(self.max_repo_size));
        // Create multi-progress bar manager
        let scanning_progress = Arc::new(MultiProgress::new());
        let mut worker_threads = Vec::with_capacity(self.worker_threads as usize);
        // Prepare worker threads
        for worker_idx in 0..self.worker_threads {
            // Calculate quota and create progress bars
            let worker_quota = self.worker_quota(worker_idx);
            let pb = ProgressBar::new(worker_quota as u64);
            pb.set_style(
                ProgressStyle::default_bar().template(
                    "[{elapsed_precise}] {bar:40.green/yellow} {pos:>7}/{len:7} {wide_msg}",
                ),
            );
            let worker_progress = scanning_progress.add(pb);
            worker_progress.enable_steady_tick(1000);
            // Get worker repo slice and commit scanning policy
            let worker_slice = Arc::new(self.worker_slice(worker_idx, trending_repo_urls.to_vec()));
            let policy = self.policy;
            // Spawn worker thread
            worker_threads.push(thread::spawn(move || {
                let mut vulnerable_code = Vec::with_capacity(worker_quota as usize);
                for git_url in worker_slice.iter() {
                    // Create temporary directory to store cloned repo
                    let repo_dir = tempdir().unwrap();
                    // Add entry to worker discovered vulnerability list if the repo contains any
                    // vulnerabilities
                    vulnerable_code.append(
                        &mut VulnerableCommits::new(
                            git_url,
                            &repo_dir,
                            Some(&worker_progress),
                            Some(policy),
                        )
                        .and_then(|vc| {
                            vc.vulnerable_code(
                                &vec![&Flawfinder::new()],
                                Some(worker_quota as usize - worker_progress.position() as usize),
                                Some(&worker_progress),
                            )
                        })
                        .unwrap_or_else(|err| {
                            worker_progress
                                .set_message(format!("Could not get vulnerable commits: {err}"));
                            vec![]
                        }),
                    );
                    // Stop cloning repositories if the worker quota is reached
                    if vulnerable_code.len() >= worker_quota as usize {
                        break;
                    }
                }
                if vulnerable_code.len() < worker_quota as usize {
                    worker_progress.abandon_with_message("Failed to reach quota");
                } else {
                    worker_progress.finish_using_style();
                }
                vulnerable_code
            }));
        }
        // Retrieve vulnerabilities from worker threads
        let mut vulnerabilities = Vec::with_capacity(self.entries as usize);
        scanning_progress
            .join_and_clear()
            .or_else(|err| Err(err.to_string()))?;
        for worker in worker_threads {
            match worker.join() {
                Ok(worker_vulnerabilities) => vulnerabilities.extend(worker_vulnerabilities),
                Err(err) => debug_print(&format!("{:?}", err)),
            }
        }
        // Create dataset
        Ok(self.generate_dataset(vulnerabilities))
    }

    /// Creates the dataset of vulnerable/benign entries based on the 
    /// settings given to the generator and saves the output as a JSON lines
    /// (`.jsonl`) file to the specified path.
    pub fn create_dataset(&self) -> Result<DataFrame, String> {
        if self.quiet {
            todo!("Quietly create vulnerability dataset")
        } else {
            self.create_dataset_verbose()
        }
    }

    /// Sets the [`WorkDivisionStrategy`] on how each worker thread will divide the
    /// workload of repos to clone and search for vulnerabilities
    pub fn set_strategy(&mut self, strategy: WorkDivisionStrategy) -> &mut Self {
        self.strategy = strategy;
        self
    }

    /// Sets the policy for worker threads on how to identify vulnerable commits.
    pub fn set_policy(&mut self, policy: VulnerableCommitIdentifier) -> &mut Self {
        self.policy = policy;
        self
    }
}
