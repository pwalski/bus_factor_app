use std::{future::Future, pin::Pin};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("General error: {0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Repo {
    fn name(&self) -> String;
}

pub struct Contributor {
    name: String,
    percentage: f32,
}

pub trait Client {
    type REPO: Repo;

    fn top_repos(
        &self,
        lang: String,
        count: f32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Self::REPO>>> + Send + '_>>;

    fn top_contributors(
        &self,
        contributor: Self::REPO,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Contributor>>> + Send + '_>>;
}
