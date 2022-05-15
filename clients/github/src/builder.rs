use crate::payload::RateLimit;
use crate::payload::RateLimitResource;
use crate::payload::RateLimitResources;
use crate::GithubClient;
use clients::api::Result;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Request;
use reqwest::Response;
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower::util::BoxService;
use tower::{service_fn, ServiceExt};

pub struct GithubClientBuilder {
    client_builder: ClientBuilder,
    github_url: String,
    headers: HeaderMap,
}

impl Default for GithubClientBuilder {
    fn default() -> Self {
        let builder = Self {
            client_builder: ClientBuilder::default(),
            github_url: "https://api.github.com".to_string(),
            headers: HeaderMap::default(),
        };
        builder
            .try_with_header(header::USER_AGENT, "curl")
            .unwrap() //TODO ugly
            .try_with_header(header::ACCEPT, "application/vnd.github.v3+json")
            .unwrap() //TODO ugly
    }
}

impl GithubClientBuilder {
    pub fn try_with_token(self, token: secrecy::SecretString) -> Result<GithubClientBuilder> {
        Ok(self.try_with_header(header::AUTHORIZATION, token.expose_secret())?)
    }

    pub fn try_with_user_agent<STR: AsRef<str>>(self, user_agent: STR) -> Result<GithubClientBuilder> {
        Ok(self.try_with_header(header::USER_AGENT, user_agent)?)
    }

    pub fn with_github_url<STR: AsRef<str>>(mut self, url: STR) -> GithubClientBuilder {
        self.github_url = url.as_ref().to_string();
        self
    }

    fn try_with_header(mut self, key: HeaderName, val: impl AsRef<str>) -> anyhow::Result<GithubClientBuilder> {
        let val = HeaderValue::from_str(val.as_ref())?;
        self.headers.insert(key, val);
        Ok(self)
    }

    pub async fn build(self) -> Result<GithubClient> {
        let client = self.client_builder.default_headers(self.headers).build()?;
        let github_url = self.github_url;
        let rate_limit = rate_limit(&client, &github_url).await?;
        let repo_service = rate_limited_service(client.clone(), &rate_limit.search)?;
        let contrib_service = rate_limited_service(client.clone(), &rate_limit.core)?;
        Ok(GithubClient {
            repo_service: Arc::new(Mutex::new(repo_service)),
            contrib_service: Arc::new(Mutex::new(contrib_service)),
            github_url,
        })
    }
}

async fn rate_limit(client: &Client, github_url: impl Into<String>) -> reqwest::Result<RateLimitResources> {
    let request_url = format!("{}/rate_limit", github_url.into());
    let response = client.get(request_url).send().await?;
    crate::read_response::<RateLimit>(response)
        .await
        .map(|resources| resources.resources)
}

fn rate_limited_service(
    client: Client,
    limit: &RateLimitResource,
) -> Result<BoxService<Request, Response, reqwest::Error>> {
    let limit = limit.limit - limit.used;
    if limit == 0 {
        return Err(clients::api::Error::Error("API rate limits reached."));
    }
    //TODO Make use from `RateLimitResource::used` field. Do not assume 1 min duration.
    let service = tower::ServiceBuilder::new()
        .rate_limit(limit as u64, Duration::from_secs(61))
        .service(service_fn(move |req| client.execute(req)))
        .boxed();
    Ok(service)
}
