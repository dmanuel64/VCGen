pub struct TrendingRepositories {
    language: String,
}

impl TrendingRepositories {
    pub fn new(language: &str) -> Self {
        Self {
            language: String::from(language),
        }
    }
}

impl Default for TrendingRepositories {
    fn default() -> Self {
        Self::new("c")
    }
}
