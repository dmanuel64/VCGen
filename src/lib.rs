use crate::dataset::vulnerabilities::save_dataset;
use dataset::vulnerabilities::create_dataset as generate_dataset;
use git::{
    search::{github_api_token, TrendingRepositories, GITHUB_API_VAR},
    vulnerability::VulnerableCommits,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{path::Path, sync::Arc, thread};
use tempfile::tempdir;
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

pub fn check_dependencies(skip_flawfinder: bool) -> Result<(), String> {
    if github_api_token().is_none() {
        Err(format!(
            "You must set the {} environment variable to your GitHub API personal access token.",
            GITHUB_API_VAR
        ))
    } else if !skip_flawfinder && Flawfinder::install_location().is_none() {
        Err(format!(
            "Flawfinder is not installed. {}{}{}",
            "Install Flawfinder with\n\nsudo apt-get install flawfinder\n\n",
            "Download it from https://dwheeler.com/flawfinder/",
            format!(
                "Or set the {} environment variable of the path to the Flawfinder executable.",
                FLAWFINDER_ENV_VAR
            )
        ))
    } else {
        Ok(())
    }
}

pub fn create_dataset(
    entries: i32,
    dataset_path: &Path,
    vulnerability_ratio: f32,
    worker_threads: i32,
    max_repo_size: Option<u32>,
) -> Result<(), String> {
    // get git URLs of trending repositories from GitHub
    let trending_repos = with_progress_spinner("Fetching popular C repositories.", || {
        TrendingRepositories::default()
    });
    let trending_git_urls = Arc::new(trending_repos.repos(max_repo_size));
    let mut vulnerabilities = vec![];
    // divide vulnerability scanning equally among worker threads
    let vulnerability_progress = Arc::new(MultiProgress::new());
    let slice_size = trending_git_urls.len() / worker_threads as usize;
    let mut slice_start = 0;
    let worker_quota = entries / worker_threads;
    let mut workers = vec![];
    for _ in 0..worker_threads {
        let trending_git_urls = Arc::clone(&trending_git_urls);
        let pb = ProgressBar::new(worker_quota as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/yellow} {pos:>7}/{len:7} {wide_msg}"),
        );
        let worker_progress = vulnerability_progress.add(pb);
        worker_progress.enable_steady_tick(1000);
        workers.push(thread::spawn(move || {
            let slice_end = if slice_start + slice_size < trending_git_urls.len() {
                slice_start + slice_size
            } else {
                trending_git_urls.len()
            };
            let mut vulnerable_code = vec![];
            for git_url in &trending_git_urls[slice_start..slice_end] {
                let repo_dir = tempdir().unwrap();
                vulnerable_code.append(
                    &mut VulnerableCommits::new(git_url, &repo_dir, Some(&worker_progress), None)
                        .and_then(|vc| {
                            vc.vulnerable_code(
                                &vec![&Flawfinder::new()],
                                Some(worker_quota as usize),
                                Some(&worker_progress),
                            )
                        })
                        .unwrap_or_else(|err| {
                            worker_progress
                                .set_message(format!("Could not get vulnerable commits: {err}"));
                            vec![]
                        }),
                );
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
        slice_start += slice_size;
    }
    // Retrieve vulnerabilities from worker threads
    vulnerability_progress
        .join_and_clear()
        .or_else(|err| Err(err.to_string()))?;
    for worker in workers {
        vulnerabilities.extend(worker.join().unwrap());
    }
    // Create dataset
    let dataset_progress = ProgressBar::new(vulnerabilities.len() as u64);
    dataset_progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green/yellow} {pos:>7}/{len:7} {wide_msg}"),
    );
    let mut df = generate_dataset(vulnerabilities, Some(&dataset_progress));
    println!("{}", &df.head(None));
    save_dataset(&mut df, dataset_path);
    dataset_progress.finish_using_style();
    println!("{:?}", df.shape());
    Ok(())
}
