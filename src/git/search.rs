use octocrab::{models::Repository, Octocrab, Page};
use std::env::var;

const GITHUB_API_VAR: &str = "GITHUB_API_TOKEN";

pub struct TrendingRepositories {
    language: String,
    github: Octocrab,
    page_number: i32,
    current_page: Page<Repository>,
}

impl TrendingRepositories {
    pub fn new(language: &str, api_token: &str) -> Result<Self, String> {
        let github = Octocrab::builder()
            .personal_token(String::from(api_token))
            .build()
            .or_else(|_| Err(String::from("Could not create GitHub API client.")))?;
        let page_number = 1;
        let current_page =
            Self::trending_repos_page(&github, language, page_number).or_else(|_| {
                Err(String::from(format!(
                    "Could not load trending {language} repositories."
                )))
            })?;
        Ok(Self {
            language: String::from(language),
            github: github,
            page_number: page_number,
            current_page: current_page,
        })
    }

    #[tokio::main]
    async fn trending_repos_page(
        github: &Octocrab,
        language: &str,
        page: i32,
    ) -> Result<Page<Repository>, octocrab::Error> {
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

    pub fn repos(&self) -> Vec<String> {
        let mut repos_urls: Vec<String> = Vec::new();
        for repo in &self.current_page {
            if let Some(git_url) = repo.git_url.clone() {
                repos_urls.push(git_url.to_string());
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
    fn default() -> Self {
        Self::new(
            "c",
            &var(GITHUB_API_VAR).expect(&format!(
                "Could not get value of environment variable {GITHUB_API_VAR}"
            )),
        )
        .expect("Could not get Trending Repositories with default settings.")
    }
}
