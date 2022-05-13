mod builder;
mod payload;

use async_trait::async_trait;
pub use builder::GithubClientBuilder;
use clients::api::Contributor;
use clients::api::Result;
use reqwest::Client;
use reqwest::Response;
use serde::de::DeserializeOwned;

pub struct GithubClient {
    client: Client,
    github_url: String,
}

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
        let response: payload::SearchRepos = read_response(response).await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, repo: &GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
        let request_url = format!("{}/repos/{}/{}/contributors", self.github_url, repo.owner, repo.name);
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
        let response: Vec<payload::Contributor> = read_response(response).await?;
        let response = response.into_iter().map(Contributor::from).collect();
        Ok(response)
    }
}

async fn read_response<PAYLOAD: DeserializeOwned>(response: Response) -> reqwest::Result<PAYLOAD> {
    let response = response.error_for_status()?;
    response.json::<PAYLOAD>().await
}
