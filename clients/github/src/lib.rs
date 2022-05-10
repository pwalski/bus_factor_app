use std::fmt::Display;

use async_trait::async_trait;
use clients::api::Contributor;
use clients::api::Result;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::ClientBuilder;
use secrecy::ExposeSecret;
use serde::Deserialize;

struct GithubClient {
    client: Client,
    github_url: String,
}

#[derive(Default)]
struct GithubClientBuilder {
    headers: HeaderMap,
    client_builder: ClientBuilder,
}

impl GithubClientBuilder {
    pub fn with_token(mut self, token: secrecy::SecretString) -> Result<GithubClientBuilder> {
        Ok(self.with_header(header::AUTHORIZATION, token.expose_secret())?)
    }

    pub fn with_user_agent(mut self, user_agent: impl AsRef<str>) -> Result<GithubClientBuilder> {
        Ok(self.with_header(header::USER_AGENT, user_agent)?)
    }

    fn with_header(mut self, key: HeaderName, val: impl AsRef<str>) -> anyhow::Result<GithubClientBuilder> {
        self.headers.insert(key, HeaderValue::from_str(val.as_ref())?);
        Ok(self)
    }

    //TODO move github_url to optional builder field, or generate builder with some derive tool
    pub fn build(mut self, github_url: String) -> Result<GithubClient> {
        self = self.with_header(header::ACCEPT, "application/vnd.github.v3+json")?;
        let client = self.client_builder.default_headers(self.headers).build()?;
        Ok(GithubClient { client, github_url })
    }
}

struct GithubRepo {
    name: String,
    owner: String,
}

impl clients::api::Repo for GithubRepo {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug)]
struct SearchReposBody {
    items: Vec<RepoBody>,
}

#[derive(Deserialize, Debug)]
struct RepoBody {
    name: String,
    owner: RepoOwnerBody,
}

#[derive(Deserialize, Debug)]
struct RepoOwnerBody {
    login: String,
}

impl From<RepoBody> for GithubRepo {
    fn from(repo: RepoBody) -> Self {
        GithubRepo {
            name: repo.name,
            owner: repo.owner.login,
        }
    }
}

#[async_trait]
impl clients::api::Client for GithubClient {
    type REPO = GithubRepo;

    async fn top_repos(&self, lang: String, page: u32, per_page: u32) -> Result<Vec<Self::REPO>> {
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
        let response = response.json::<SearchReposBody>().await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, contributor: Self::REPO, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use crate::GithubClientBuilder;
    use clients::api::Client;

    // #[tokio::test]
    async fn naive_bad_test() {
        let client = GithubClientBuilder::default()
            .with_user_agent("curl")
            .unwrap()
            .build("https://api.github.com".to_string())
            .unwrap();
        let res = client.top_repos("rust".to_string(), 1, 25).await.unwrap();
        assert!(res.len() == 25);
        assert_eq!(res[0].name, "deno");
    }
}
