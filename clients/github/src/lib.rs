mod builder;
mod limiter;
mod payload;

use async_trait::async_trait;
use bus_factor::api::Contributor;
use bus_factor::api::Result;
use derive_more::Constructor;
use limiter::RateLimiter;
use reqwest::Client;
use reqwest::Response;
use serde::de::DeserializeOwned;

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
    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<GithubRepo>> {
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
            ])
            .send()
            .await?;
        self.repos_limiter.reset_limiter(&response.headers()).await?;
        let response: payload::SearchRepos = read_response(response).await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, repo: &GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
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
