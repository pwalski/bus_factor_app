mod builder;
mod payload;

pub use builder::GithubClientBuilder;

use async_trait::async_trait;
use clients::api::Contributor;
use clients::api::Result;
use reqwest::Client;

pub struct GithubClient {
    client: Client,
    github_url: String,
}

pub struct GithubRepo {
    name: String,
    owner: String,
}

impl clients::api::Repo for GithubRepo {
    fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl clients::api::Client<GithubRepo, 100, 100> for GithubClient {
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
        let response = response.json::<payload::SearchRepos>().await?;
        let response = response.items.into_iter().map(GithubRepo::from).collect();
        Ok(response)
    }

    async fn top_contributors(&self, repo: GithubRepo, page: u32, per_page: u32) -> Result<Vec<Contributor>> {
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
        let response = response.json::<Vec<payload::Contributor>>().await?;
        let response = response.into_iter().map(Contributor::from).collect();
        Ok(response)
    }
}
#[cfg(test)]
mod tests {
    use crate::{GithubClientBuilder, GithubRepo};
    use clients::api::Client;

    // #[tokio::test]
    async fn naive_bad_top_repos_test() -> Result<(), anyhow::Error> {
        let client = GithubClientBuilder::default().try_with_user_agent("curl")?.build()?;
        let res = client.top_repos("rust".to_string(), 1, 25).await?;
        assert!(res.len() == 25);
        assert_eq!(res[0].name, "deno");
        Ok(())
    }

    // #[tokio::test]
    async fn naive_bad_top_contributors_test() -> Result<(), anyhow::Error> {
        let client = GithubClientBuilder::default().try_with_user_agent("curl")?.build()?;
        let repo = GithubRepo {
            name: "deno".into(),
            owner: "denoland".into(),
        };
        let res = client.top_contributors(repo, 1, 5).await?;
        assert!(res.len() == 5);
        assert_eq!(res[0].name, "ry");
        assert_eq!(res[0].contributions, 1396);
        Ok(())
    }
}
