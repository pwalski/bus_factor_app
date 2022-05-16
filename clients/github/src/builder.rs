use crate::limiter::RateLimit;
use crate::limiter::RateLimiter;
use crate::payload::RateLimitBody;
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
use secrecy::ExposeSecret;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        let rate_limit = get_rate_limit(&client, &github_url).await?;
        let repos_limiter = rate_limit.search.into();
        let contrib_limiter = rate_limit.core.into();
        Ok(GithubClient::new(client, github_url, repos_limiter, contrib_limiter))
    }
}

async fn get_rate_limit(client: &Client, github_url: impl Into<String>) -> reqwest::Result<RateLimitResources> {
    let request_url = format!("{}/rate_limit", github_url.into());
    let response = client.get(request_url).send().await?;
    crate::read_response::<RateLimitBody>(response)
        .await
        .map(|resources| resources.resources)
}

impl From<RateLimitResource> for RateLimiter {
    fn from(limit_resource: RateLimitResource) -> Self {
        let remaining = std::cmp::max(limit_resource.remaining, 1) - 1;
        let limit = RateLimit::new(limit_resource.limit, remaining, limit_resource.reset);
        RateLimiter::new(Arc::new(Mutex::new(limit)))
    }
}
