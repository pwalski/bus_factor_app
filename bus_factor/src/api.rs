use async_trait::async_trait;
use std::fmt::{Debug, Display};
use strum_macros::{AsRefStr, EnumString};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Error(String),
    // the only reason of `reqwest` dependency..
    #[error("Request error: {0}")]
    RequestError(String),
    // the only reason of `reqwest` dependency..
    #[error("Client error: {0}")]
    ClientError(#[from] anyhow::Error),
    // the only reason of `reqwest` dependency..
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
    async fn top_repos(&self, lang: String, page: u32, per_page: u32, order: Sort) -> Result<Vec<REPO>>;

    async fn top_contributors(&self, contributor: &'_ REPO, page: u32, per_page: u32) -> Result<Vec<Contributor>>;
}

#[derive(Debug, EnumString, Clone, AsRefStr)]
pub enum Sort {
    #[strum(serialize = "stars")]
    Stars,
    #[strum(serialize = "forks")]
    Forks,
    #[strum(serialize = "help_wanted_issues")]
    HelpWantedIssues,
    #[strum(serialize = "udpated")]
    Updated,
}
