use git::{search::TrendingRepositories, vulnerability::VulnerableCommits};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    path::Path,
    sync::Arc,
    thread::{self, sleep},
};
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
    max_repo_size: Option<u32>,
) -> Result<(), String> {
    // get git URLs of trending repositories from GitHub
    let trending_repos = with_progress_spinner("Fetching popular C repositories.", || {
        TrendingRepositories::default()
    });
    let trending_git_urls = Arc::new(trending_repos.repos(max_repo_size));
    // divide vulnerability scanning among worker threads
    let vulnerability_progress = Arc::new(MultiProgress::new());
    let slice_size = trending_git_urls.len() / worker_threads as usize;
    let mut slice_start = 0;
    let worker_quota = entries / worker_threads;
    let mut workers = vec![];
    for worker in 0..worker_threads {
        let trending_git_urls = Arc::clone(&trending_git_urls);
        let pb = ProgressBar::new(worker_quota as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/yellow} {pos:>7}/{len:7} {wide_msg}"),
        );
        let worker_progress = vulnerability_progress.add(pb);
        workers.push(thread::spawn(move || {
            let slice_end = if slice_start + slice_size < trending_git_urls.len() {
                slice_start + slice_size
            } else {
                trending_git_urls.len()
            };
            let mut vulnerable_code = vec![];
            for git_url in &trending_git_urls[slice_start..slice_end] {
                vulnerable_code.append(
                    &mut VulnerableCommits::new(git_url, Some(&worker_progress))
                        .and_then(|vc| {
                            vc.vulnerable_code(
                                vec![Box::new(Flawfinder::default())],
                                Some(worker_quota),
                                Some(&worker_progress),
                            )
                        })
                        .unwrap_or_else(|err| {
                            worker_progress
                                .set_message(format!("Could not find vulnerable commits: {err}"));
                            sleep(core::time::Duration::from_millis(3000));
                            vec![]
                        }),
                );
                break;
            }
            worker_progress.finish_using_style();
        }));
        slice_start += slice_size;
    }
    vulnerability_progress
        .join_and_clear()
        .or_else(|err| Err(err.to_string()))?;
    for worker in workers {
        worker.join().unwrap();
    }
    // for git_url in trending_repos.repos() {
    //     let commits = VulnerableCommits::new(&git_url, vec![Box::new(Flawfinder)]);
    //     break;
    // }
    Ok(())
}
