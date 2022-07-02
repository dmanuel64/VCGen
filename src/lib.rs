use git::search::TrendingRepositories;
use std::path::Path;

mod git;

pub fn create_dataset(
    entries: i32,
    dataset_path: &Path,
    vulnerability_ratio: f32,
    quiet: bool,
) -> Result<(), String> {
    let trending_repos = TrendingRepositories::default();
    Ok(())
}
