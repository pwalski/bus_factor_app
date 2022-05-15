//! Bus factor estimation
//!
//! # Overview
//!
//! Bus factor is a measurement which attempts to estimate the number of key persons a repository would need to lose in order for it to become stalled due to lack of expertise.
//! It is commonly used in the context of software development.
//! For example, if a given repository is developed by a single person, then the repository's bus factor is equal to 1 (it's likely for the repository to become unmaintained if the main contributor suddenly stops working on it).
//!
//! Library finds popular GitHub repositories with a bus factor of 1.
//! Given a programming language name (`language`) and a repository count (`repo_count`), library fetches the first `repo_count` most popular repositories (sorted by the number of GitHub stars) from the given language.
//! Then, for each repository, it inspect its contributor statistics.
//! We assume a repository's bus factor is 1 if its most active developer's contributions account for 75% or more of the total contributions count from the top 25 most active developers.
//! Repositories with a bus factor of 75% or higher are returned as a Result.

use std::fmt::Debug;
use std::pin::Pin;
use std::{fmt::Display, marker::PhantomData, sync::Arc};

use clients::api::{Client, Contributor, Repo};
use derive_more::Constructor;
use futures::task::Poll;
use futures::{future, stream, Stream, StreamExt, TryStreamExt};
use log::{debug, error};
use tokio::task::JoinHandle;

#[derive(Debug, PartialEq, Constructor)]
pub struct BusFactor {
    repo: String,
    contributor: String,
    bus_factor: f32,
}

impl Display for BusFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "project: {}\tuser: {}\tpercentage: {}",
            self.repo, self.contributor, self.bus_factor
        ))
    }
}

#[derive(Constructor)]
struct Page {
    page_no: u32,
    page_size: u32,
}
#[derive(Constructor)]
struct Paginator {
    page_no: u32,
    max_page_size: u32,
    remaining: u32,
}

impl Paginator {
    fn next_page(&mut self) -> Option<Page> {
        let page_no = self.page_no;
        self.page_no = self.page_no + 1;
        match self.remaining {
            0 => None,
            remaining if remaining <= self.max_page_size => {
                self.remaining = 0;
                Some(Page::new(page_no, remaining))
            }
            r => {
                self.remaining = self.remaining - self.max_page_size;
                Some(Page::new(page_no, self.max_page_size))
            }
        }
    }
}

pub type BusFactorStream = Pin<Box<dyn Stream<Item = BusFactor> + std::marker::Send>>;
pub struct BusFactorCalculator<
    REPO,
    const MAX_REPOS_PAGE: u32,
    const MAX_CONTRIBUTORS_PAGE: u32,
    const FIRST_PAGE_NUMBER: u32,
    CLIENT,
> where
    REPO: Repo,
    CLIENT: 'static + Client<REPO, MAX_REPOS_PAGE, MAX_CONTRIBUTORS_PAGE, FIRST_PAGE_NUMBER>,
{
    client: Arc<CLIENT>,
    threshold: f32,
    _repo_type: PhantomData<REPO>,
}

impl<REPO, const MAX_REPOS_PAGE: u32, const MAX_CONTRIBUTORS_PAGE: u32, const FIRST_PAGE_NUMBER: u32, CLIENT>
    BusFactorCalculator<REPO, MAX_REPOS_PAGE, MAX_CONTRIBUTORS_PAGE, FIRST_PAGE_NUMBER, CLIENT>
where
    REPO: 'static + Repo,
    CLIENT: 'static + Client<REPO, MAX_REPOS_PAGE, MAX_CONTRIBUTORS_PAGE, FIRST_PAGE_NUMBER>,
{
    pub fn new(client: CLIENT, threshold: f32) -> Self {
        let _repo_type = PhantomData::default();
        BusFactorCalculator {
            client: Arc::new(client),
            threshold,
            _repo_type,
        }
    }

    pub fn calculate(self, lang: String, repo_count: u32) -> BusFactorStream {
        Self::top_repos(self.client.clone(), lang, repo_count)
            .buffered(1)
            .flat_map(|repos| {
                match repos {
                    Ok(Ok(repos)) => stream::iter(repos),
                    err => {
                        error!("Failed to get top repositories: {:?}", err);
                        stream::iter(Vec::new()) //TODO how to return stream::empty() ???
                    }
                }
            })
            .map(move |r| Self::repo_bus_factor(r, self.client.clone(), self.threshold))
            .buffered(10)
            .filter_map(|bus_factor| async move {
                match bus_factor {
                    Ok(bus_factor) => bus_factor,
                    err => {
                        error!("Failed to calculate bus factor: {:?}", err);
                        None
                    }
                }
            })
            .boxed()
    }

    fn top_repos(
        client: Arc<CLIENT>,
        lang: String,
        repo_count: u32,
    ) -> Pin<Box<dyn Stream<Item = JoinHandle<Result<Vec<REPO>, clients::api::Error>>> + Send>> {
        let mut paginator = Paginator {
            max_page_size: MAX_REPOS_PAGE,
            page_no: FIRST_PAGE_NUMBER,
            remaining: repo_count,
        };
        stream::poll_fn(move |_| Poll::Ready(paginator.next_page()))
            .map(move |page| {
                let client = client.clone();
                let lang = lang.clone();
                tokio::spawn(Self::top_repos_page(client, lang, page))
            })
            .boxed()
    }

    async fn top_repos_page(client: Arc<CLIENT>, lang: String, page: Page) -> clients::api::Result<Vec<REPO>> {
        if page.page_size < MAX_REPOS_PAGE {
            if page.page_no == FIRST_PAGE_NUMBER {
                client.top_repos(lang.into(), page.page_no, page.page_size).await
            } else {
                let repos = client.top_repos(lang.into(), page.page_no, MAX_REPOS_PAGE).await;
                repos.map(|v| Self::take_first_n(v, page.page_size))
            }
        } else {
            client.top_repos(lang.into(), page.page_no, page.page_size).await
        }
    }

    fn take_first_n<T>(v: Vec<T>, n: u32) -> Vec<T> {
        v.into_iter().take(n as usize).collect()
    }

    fn repo_bus_factor(repo: REPO, client: Arc<CLIENT>, threshold: f32) -> JoinHandle<Option<BusFactor>> {
        // TODO add parameter for 'per_page'
        let client = client.clone();
        tokio::spawn(async move {
            client
                .top_contributors(&repo, FIRST_PAGE_NUMBER, 25)
                .await
                .map(|contributors| contributors_bus_factor(contributors, repo.name().into(), threshold))
                .unwrap_or_else(|err| {
                    error!("Failed to get top contributors: {}", err);
                    None
                })
        })
    }
}

/// Returns `RepoBusFactor` if `threshold` reached.
///
/// # Arguments
/// * `contributors` - List of `Contributor`s sorted by contributions in desc order
/// * `repo` - Name of repository
/// * `threshold` - contribution ratio threshold of top(first) contributor to total contributions of listed `contributors`
fn contributors_bus_factor(contributors: Vec<Contributor>, repo: String, threashold: f32) -> Option<BusFactor> {
    let top_contributor = contributors.get(0)?;
    let total_contributions = contributors
        .iter()
        .map(|contributor| contributor.contributions)
        .fold(0, |acc, c| acc + c);
    let bus_factor = calculate_percentage(top_contributor.contributions, total_contributions);
    if bus_factor >= threashold {
        Some(BusFactor::new(repo, top_contributor.name.to_string(), bus_factor))
    } else {
        None
    }
}

/// Produces float from range [0.0,1.1] rounded to two decimal points.
fn calculate_percentage(contributions: u32, total_contributions: u32) -> f32 {
    let bus_factor = contributions as f32 / total_contributions as f32;
    (&format!("{0:.1$}", bus_factor, 2)).parse().unwrap() //TODO probably there is a smarter way to do this...
}

#[test]
fn bus_factor_some_test() {
    let contributors = vec![
        Contributor::new("a", 7),
        Contributor::new("b", 2),
        Contributor::new("c", 1),
    ];
    let repo = "repo".to_string();
    let bus_factor = contributors_bus_factor(contributors, repo.clone(), 0.6);
    assert_eq!(bus_factor, Some(BusFactor::new(repo, "a".to_string(), 0.7)));
}

#[test]
fn bus_factor_none_test() {
    let contributors = vec![
        Contributor::new("a", 7),
        Contributor::new("b", 2),
        Contributor::new("c", 1),
    ];
    let repo = "repo".to_string();
    let bus_factor = contributors_bus_factor(contributors, repo.clone(), 0.8);
    assert_eq!(bus_factor, None);
}

#[test]
fn bus_factor_onedev_test() {
    let contributors = vec![Contributor::new("a", 7)];
    let repo = "repo".to_string();
    let bus_factor = contributors_bus_factor(contributors, repo.clone(), 0.99);
    assert_eq!(bus_factor, Some(BusFactor::new(repo, "a".to_string(), 1.0)));
}
