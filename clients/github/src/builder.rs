use crate::payload::RateLimitBody;
use crate::payload::RateLimitResource;
use crate::payload::RateLimitResources;
use crate::GithubClient;
use crate::RateLimit;
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
        let rate_limit = rate_limit(&client, &github_url).await?;
        let repos_limiter = Arc::new(Mutex::new(rate_limit.search.into()));
        let contrib_limiter = Arc::new(Mutex::new(rate_limit.core.into()));
        Ok(GithubClient {
            client,
            github_url,
            repos_limiter,
            contrib_limiter,
        })
    }
}

async fn rate_limit(client: &Client, github_url: impl Into<String>) -> reqwest::Result<RateLimitResources> {
    let request_url = format!("{}/rate_limit", github_url.into());
    let response = client.get(request_url).send().await?;
    crate::read_response::<RateLimitBody>(response)
        .await
        .map(|resources| resources.resources)
}

impl From<RateLimitResource> for RateLimit {
    fn from(rate_limit: RateLimitResource) -> Self {
        RateLimit {
            limit: rate_limit.limit,
            remaining: rate_limit.remaining - 1,
            reset: rate_limit.reset,
        }
    }
}
