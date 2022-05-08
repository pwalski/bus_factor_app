use clients::api::Contributor;
use clients::api::Result;
use std::future::Future;
use std::pin::Pin;
struct GithubClient {}

struct GithubRepo {}

impl clients::api::Repo for GithubRepo {
    fn name(&self) -> String {
        todo!()
    }
}

impl clients::api::Client for GithubClient {
    type REPO = GithubRepo;

    fn top_repos(
        &self,
        lang: String,
        count: f32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Self::REPO>>> + Send + '_>> {
        todo!()
    }

    fn top_contributors(
        &self,
        contributor: Self::REPO,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Contributor>>> + Send + '_>> {
        todo!()
    }
}
