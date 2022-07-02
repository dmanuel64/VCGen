use git::{search::TrendingRepositories, vulnerability::VulnerableCommits};
use vulnerability::tools::Flawfinder;
use std::path::Path;

mod git;
mod vulnerability;

pub fn create_dataset(
    entries: i32,
    dataset_path: &Path,
    vulnerability_ratio: f32,
    quiet: bool,
) -> Result<(), String> {
    let trending_repos = TrendingRepositories::default();
    for git_url in trending_repos.repos() {
        let commits = VulnerableCommits::new(&git_url, vec![Box::new(Flawfinder)]);
        assert!(commits.unwrap().repo_dir.path().exists());
        break;
    }
    Ok(())
}
