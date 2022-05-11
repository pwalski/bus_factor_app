use clients::api::Result;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::ClientBuilder;
use secrecy::ExposeSecret;

use crate::GithubClient;
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

    pub fn build(self) -> Result<GithubClient> {
        let client = self.client_builder.default_headers(self.headers).build()?;
        let github_url = self.github_url;
        Ok(GithubClient { client, github_url })
    }
}
