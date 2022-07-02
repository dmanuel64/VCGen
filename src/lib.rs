use git::{search::TrendingRepositories, vulnerability::VulnerableCommits};
use indicatif::ProgressBar;
use std::{path::Path, thread};
use vulnerability::tools::Flawfinder;

mod git;
mod vulnerability;

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

pub fn create_dataset(
    entries: i32,
    dataset_path: &Path,
    vulnerability_ratio: f32,
    worker_threads: i32,
) -> Result<(), String> {
    // get git URLs of trending repositories from GitHub
    let trending_repos = with_progress_spinner("Fetching popular C repositories.", || {
        TrendingRepositories::default()
    });
    let trending_git_urls = trending_repos.repos();
    // divide among worker threads
    let slice_size = trending_git_urls.len() / worker_threads as usize;
    let mut workers = vec![];
    for worker in 0..worker_threads {
        workers.push(thread::spawn(|| 1));
    }
    let f = &trending_git_urls[0..2];
    // for git_url in trending_repos.repos() {
    //     let commits = VulnerableCommits::new(&git_url, vec![Box::new(Flawfinder)]);
    //     break;
    // }
    Ok(())
}
