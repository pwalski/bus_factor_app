use async_trait::async_trait;
use std::{fmt::Display, future::Future, pin::Pin};
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

pub trait Repo {
    fn name(&self) -> &str;
}

pub struct Contributor {
    pub name: String,
    pub contributions: u32,
}

#[async_trait]
pub trait Client {
    type REPO: Repo + Send + Sync;

    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<Self::REPO>>;

    async fn top_contributors(&self, contributor: Self::REPO, page: u32, per_page: u32) -> Result<Vec<Contributor>>;
}
