mod builder;
mod limiter;
mod payload;

use async_trait::async_trait;
use bus_factor::api::Contributor;
use bus_factor::api::Sort;
use derive_more::Constructor;
use limiter::RateLimiter;
use reqwest::Client;
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::convert::AsRef;
use thiserror::Error;

pub use builder::GithubClientBuilder;

#[derive(Constructor)]
pub struct GithubClient {
    client: Client,
    github_url: String,
    repos_limiter: RateLimiter,
    contrib_limiter: RateLimiter,
}

#[derive(Debug)]
pub struct GithubRepo {
    name: String,
    owner: String,
}

impl bus_factor::api::Repo for GithubRepo {
    type T = String;
    fn name(&self) -> Self::T {
        self.name.clone()
    }
}

#[async_trait]
impl bus_factor::api::Client<GithubRepo, 100, 100, 1> for GithubClient {
    async fn top_repos(
        &self,
        lang: String,
        page: u32,
        per_page: u32,
        order: Sort,
    ) -> bus_factor::api::Result<Vec<GithubRepo>> {
        self.get_top_repos(lang, page, per_page, order)
            .await
            .map_err(crate::Error::into)
    }

    async fn top_contributors(
        &self,
        repo: &GithubRepo,
        page: u32,
        per_page: u32,
    ) -> bus_factor::api::Result<Vec<Contributor>> {
        self.get_top_contributors(repo, page, per_page)
            .await
            .map_err(crate::Error::into)
    }
}

impl GithubClient {
    async fn get_top_repos(&self, lang: String, page: u32, per_page: u32, order: Sort) -> Result<Vec<GithubRepo>> {
        let request_url = format!("{}/search/repositories", self.github_url);
        let lang_query = format!("language:{}", lang);
        self.repos_limiter.wait().await;
        let response = self
            .client
            .get(request_url)
            .query(&[
                ("q", lang_query),
                ("sort", "stars".to_string()),
                ("order", "desc".to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
                ("sort", String::from(order.as_ref())),
            ])
            .send()
            .await?;
        self.repos_limiter.reset_limiter(&response.headers()).await?;
        let response: payload::SearchRepos = read_response(response).await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn get_top_contributors(&self, repo: &GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
        let request_url = format!("{}/repos/{}/{}/contributors", self.github_url, repo.owner, repo.name);
        self.contrib_limiter.wait().await;
        let response = self
            .client
            .get(request_url)
            .query(&[
                ("anon", false.to_string()), //TODO check if `true` will produce empty names
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
            .send()
            .await?;
        self.contrib_limiter.reset_limiter(response.headers()).await?;
        let response: Vec<payload::Contributor> = read_response(response).await?;
        let response = response.into_iter().map(Contributor::from).collect();
        Ok(response)
    }
}

async fn read_response<PAYLOAD: DeserializeOwned>(response: Response) -> reqwest::Result<PAYLOAD> {
    let response = response.error_for_status()?;
    response.json::<PAYLOAD>().await
}

// Result and Errors

pub(crate) type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Error: {0}")]
    Error(String),
    #[error("Request error: {}", .0.status().map_or_else(|| "Unknown".to_string(), |status| status.to_string()))] //meh
    RequestError(#[from] reqwest::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Header parse error: {0}")]
    HeaderParseError(#[from] reqwest::header::ToStrError),
    #[error("Header value parse error: {0}")]
    HeaderValueParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Into<bus_factor::api::Error> for Error {
    fn into(self) -> bus_factor::api::Error {
        match self {
            err @ Self::RequestError(_) => bus_factor::api::Error::RequestError(err.to_string()),
            err => bus_factor::api::Error::Error(err.to_string()),
        }
    }
}

//TODO do it using `thiserror`
impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Error(msg)
    }
}
