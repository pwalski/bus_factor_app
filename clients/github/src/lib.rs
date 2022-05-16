mod builder;
mod payload;

use async_trait::async_trait;
use chrono::Utc;
use clients::api::Contributor;
use clients::api::Result;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub use builder::GithubClientBuilder;

pub struct GithubClient {
    client: Client,
    github_url: String,
    repos_limiter: Arc<Mutex<RateLimit>>,
    contrib_limiter: Arc<Mutex<RateLimit>>,
}

pub struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: i64,
}

#[derive(Debug)]
pub struct GithubRepo {
    name: String,
    owner: String,
}

impl clients::api::Repo for GithubRepo {
    type T = String;
    fn name(&self) -> Self::T {
        self.name.clone()
    }
}

#[async_trait]
impl clients::api::Client<GithubRepo, 100, 100, 1> for GithubClient {
    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<GithubRepo>> {
        let request_url = format!("{}/search/repositories", self.github_url);
        let lang_query = format!("language:{}", lang);
        wait(&self.repos_limiter).await;
        let response = self
            .client
            .get(request_url)
            .query(&[
                ("q", lang_query),
                ("sort", "stars".to_string()),
                ("order", "desc".to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
            .send()
            .await?;
        reset_limiter(&self.repos_limiter, &response.headers()).await?;
        let response: payload::SearchRepos = read_response(response).await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, repo: &GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
        let request_url = format!("{}/repos/{}/{}/contributors", self.github_url, repo.owner, repo.name);
        wait(&self.contrib_limiter).await;
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
        reset_limiter(&self.contrib_limiter, response.headers()).await?;
        let response: Vec<payload::Contributor> = read_response(response).await?;
        let response = response.into_iter().map(Contributor::from).collect();
        Ok(response)
    }
}

async fn read_response<PAYLOAD: DeserializeOwned>(response: Response) -> reqwest::Result<PAYLOAD> {
    let response = response.error_for_status()?;
    response.json::<PAYLOAD>().await
}

async fn wait(rate_limiter: &Arc<Mutex<RateLimit>>) {
    while let Some(delay) = time_to_wait(&rate_limiter).await {
        debug!("Rate limiting wait: {}", delay.as_secs());
        tokio::time::sleep(delay).await;
    }
}

async fn time_to_wait(rate_limiter: &Arc<Mutex<RateLimit>>) -> Option<Duration> {
    let mut rate_limiter = rate_limiter.lock().await;
    if rate_limiter.remaining > 0 {
        rate_limiter.remaining = rate_limiter.remaining - 1;
        return None;
    }
    let now = Utc::now().timestamp();
    if rate_limiter.reset < now {
        //TODO API limit could change so maybe should GET /rate_limit
        rate_limiter.remaining = rate_limiter.limit - 1;
        return None;
    }
    Some(Duration::new(rate_limiter.reset as u64 - now as u64, 0))
}

async fn reset_limiter(rate_limiter: &Arc<Mutex<RateLimit>>, headers: &HeaderMap<HeaderValue>) -> Result<()> {
    let mut rate_limiter = rate_limiter.lock().await;
    rate_limiter.limit = read_header::<u32>(headers, "x-ratelimit-limit")?;
    rate_limiter.remaining = read_header::<u32>(headers, "x-ratelimit-remaining")?;
    rate_limiter.reset = read_header::<i64>(headers, "x-ratelimit-reset")?;
    Ok(())
}

fn read_header<T>(headers: &HeaderMap<HeaderValue>, header: &str) -> Result<T>
where
    T: FromStr,
    clients::api::Error: From<<T as FromStr>::Err>,
{
    let header = headers
        .get(header)
        .ok_or_else(|| format!("Header {} not found", header.to_string()))
        .map(HeaderValue::to_str)??;
    Ok(header.parse::<T>()?)
}
