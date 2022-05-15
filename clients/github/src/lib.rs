mod builder;
mod payload;

use async_trait::async_trait;
use clients::api::Contributor;
use clients::api::Result;
use reqwest::Method;
use reqwest::Request;
use reqwest::Response;
use reqwest::Url;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::util::BoxService;
use tower::{Service, ServiceExt};

pub use builder::GithubClientBuilder;

pub struct GithubClient {
    repo_service: Arc<Mutex<BoxService<Request, Response, reqwest::Error>>>,
    contrib_service: Arc<Mutex<BoxService<Request, Response, reqwest::Error>>>,
    github_url: String,
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
        let url = Url::parse_with_params(
            &request_url,
            &[
                ("q", lang_query),
                ("sort", "stars".to_string()),
                ("order", "desc".to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )?;
        let request = Request::new(Method::GET, url);
        // let response = self.repo_service.ready().await?.call(request).await?;
        let response = call_service(self.repo_service.clone(), request).await?;
        let response: payload::SearchRepos = read_response(response).await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, repo: &GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
        let request_url = format!("{}/repos/{}/{}/contributors", self.github_url, repo.owner, repo.name);
        let url = Url::parse_with_params(
            &request_url,
            &[
                ("anon", false.to_string()), //TODO check if `true` will produce empty names
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )?;
        let request = Request::new(Method::GET, url);
        // let response = self.contrib_service.ready().await?.call(request).await?;
        let response = call_service(self.contrib_service.clone(), request).await?;
        let response: Vec<payload::Contributor> = read_response(response).await?;
        let response = response.into_iter().map(Contributor::from).collect();
        Ok(response)
    }
}

async fn call_service(
    service: Arc<Mutex<BoxService<Request, Response, reqwest::Error>>>,
    request: Request,
) -> Result<Response> {
    let mut service = service.lock().await;
    let response = service.ready().await?.call(request).await?;
    Ok(response)
}

async fn read_response<PAYLOAD: DeserializeOwned>(response: Response) -> reqwest::Result<PAYLOAD> {
    let response = response.error_for_status()?;
    response.json::<PAYLOAD>().await
}
