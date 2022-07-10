use octocrab::{models::Repository, Octocrab, Page};
use std::env::var;

pub const GITHUB_API_VAR: &str = "GITHUB_API_TOKEN";

/// Gets the user's GitHub API token by retrieving the value
/// set by environment variable [`GITHUB_API_VAR`].
pub fn github_api_token() -> Option<String> {
    var(GITHUB_API_VAR).ok()
}

/// Structure containing functionality to collect popular repositories of
/// a specified language.
pub struct TrendingRepositories {
    language: String,
    github: Octocrab,
    page_number: i32,
    current_page: Page<Repository>,
}

impl TrendingRepositories {
    /// Gathers the first page of trending repositories from GitHub.com
    /// using a specified language.
    pub fn new(language: &str) -> Result<Self, String> {
        // Build GitHub API client using a personal access token
        let access_token = github_api_token().ok_or(String::from(
            "Could not get GitHub API personal access token.",
        ))?;
        let github = Octocrab::builder()
            .personal_token(access_token)
            .build()
            .or_else(|_| Err(String::from("Could not create GitHub API client.")))?;
        // Get trending repositories from the first page of trending repos
        let page_number = 1;
        let current_page = Self::trending_repos_page(&github, language, page_number)
            .or_else(|_| Err(format!("Could not load trending {language} repositories.")))?;
        Ok(Self {
            language: String::from(language),
            github: github,
            page_number: page_number,
            current_page: current_page,
        })
    }

    /// Gets the top trending repositories of a language using the GitHub
    /// API client and specifying which page of results to collect from.
    #[tokio::main]
    async fn trending_repos_page(
        github: &Octocrab,
        language: &str,
        page: i32,
    ) -> Result<Page<Repository>, octocrab::Error> {
        // Search repositories using the specified language, sorting by
        // most stars (rating), in descending order (from most popular to least popular),
        // with 100 results (API limit) per page
        github
            .search()
            .repositories(&format!("language:{language}"))
            .sort("stars")
            .order("desc")
            .page(page as u32)
            .per_page(100)
            .send()
            .await
    }

    /// Gets the GitHub URLs of trending repositories of the set language,
    /// optionally limiting the results to repositories under a certain
    /// amount of KBs.
    pub fn repos(&self, size_limit: Option<u32>) -> Vec<String> {
        let mut repos_urls: Vec<String> = Vec::new();
        for repo in &self.current_page {
            // Check if repository size is under the specified amount
            if repo.size.unwrap_or_default() < size_limit.unwrap_or_else(|| u32::MAX) {
                if let Some(git_url) = &repo.clone_url {
                    // Add URL
                    repos_urls.push(git_url.to_string());
                }
            }
        }
        repos_urls
    }

    pub fn more_repos(&mut self) -> Result<(), String> {
        if let Ok(next_page) =
            Self::trending_repos_page(&self.github, &self.language, self.page_number)
        {
            self.page_number += 1;
            self.current_page = next_page;
            Ok(())
        } else {
            Err(String::from("No more pages to collect repositories."))
        }
    }
}

impl Default for TrendingRepositories {
    /// Collects trending C repositories from GitHub
    fn default() -> Self {
        Self::new("c")
            .expect("Could not get trending C repositories from GitHub.com with default settings.")
    }
}
