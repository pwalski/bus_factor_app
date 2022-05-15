use async_trait::async_trait;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Error(&'static str),
    // the only reason of `reqwest` dependency..
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Repo: Send + Sync + Debug {
    type T: Into<String> + Display;
    fn name(&self) -> Self::T;
}

pub struct Contributor {
    pub name: String,
    pub contributions: u32,
}

impl Contributor {
    pub fn new(name: impl Into<String>, contributions: u32) -> Self {
        Contributor {
            name: name.into(),
            contributions,
        }
    }
}

// TODO Just realized exposing `per_page` is dumb, because there is no point in changing it after first page.
#[async_trait]
pub trait Client<REPO: Repo, const MAX_REPOS_PAGE: u32, const MAX_CONTRIBUTORS_PAGE: u32, const FIRST_PAGE_NUMBER: u32>:
    Send + Sync
{
    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<REPO>>;

    async fn top_contributors(&self, contributor: &'_ REPO, page: u32, per_page: u32) -> Result<Vec<Contributor>>;
}
