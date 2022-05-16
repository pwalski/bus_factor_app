use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchRepos {
    pub items: Vec<Repo>,
}

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub name: String,
    pub owner: RepoOwner,
}

#[derive(Deserialize, Debug)]
pub struct RepoOwner {
    pub login: String,
}

impl From<Repo> for crate::GithubRepo {
    fn from(repo: Repo) -> Self {
        crate::GithubRepo {
            name: repo.name,
            owner: repo.owner.login,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Contributor {
    pub login: String,
    pub contributions: u32,
}

impl From<Contributor> for clients::api::Contributor {
    fn from(contributor: Contributor) -> Self {
        clients::api::Contributor {
            name: contributor.login,
            contributions: contributor.contributions,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RateLimitBody {
    pub resources: RateLimitResources,
}

#[derive(Deserialize, Debug)]
pub struct RateLimitResources {
    pub core: RateLimitResource,
    pub search: RateLimitResource,
}

#[derive(Deserialize, Debug)]
pub struct RateLimitResource {
    pub limit: u32,
    pub remaining: u32,
    pub reset: i64,
}
