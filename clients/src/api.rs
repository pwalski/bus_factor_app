use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Error(&'static str),
    // the only reason of `reqwest` dependency..
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Repo: Send + Sync {
    fn name(&self) -> &str;
}

pub struct Contributor {
    pub name: String,
    pub contributions: u32,
}

#[async_trait]
pub trait Client<REPO: Repo, const MAX_REPOS_PAGE: u32, const MAX_CONTRIBUTORS_PAGE: u32>:
    Send + Sync
{
    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<REPO>>;

    async fn top_contributors(
        &self,
        contributor: REPO,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<Contributor>>;
}
