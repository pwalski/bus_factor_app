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
//! repositories with a bus factor of 75% or higher are returned as a Result.

use std::fmt::Debug;
use std::{fmt::Display, marker::PhantomData, sync::Arc};

use clients::api::{Client, Contributor, Repo};
use derive_new::new;
use futures::{stream, StreamExt};
use log::{debug, error};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug, PartialEq, new)]
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
        let client = Arc::new(client);
        let _repo_type = PhantomData::default();
        BusFactorCalculator {
            client,
            threshold,
            _repo_type,
        }
    }

    pub fn calculate(&self, lang: String, repo_count: u32) -> Receiver<BusFactor> {
        let repo_receiver = Self::top_repos(self.client.clone(), lang, repo_count);
        let bus_factor_receiver = Self::top_contributors(repo_receiver, self.client.clone(), self.threshold);
        return bus_factor_receiver;
    }

    fn top_repos(client: Arc<CLIENT>, lang: String, mut repo_count: u32) -> Receiver<Vec<REPO>> {
        let (sender, receiver) = tokio::sync::mpsc::channel::<Vec<REPO>>(1);
        let mut page_num = FIRST_PAGE_NUMBER;
        tokio::spawn(async move {
            while repo_count > 0 {
                match Self::top_repos_page(&client, &lang, repo_count, page_num).await {
                    Ok(repos) => {
                        debug!("Got {} repositories", repos.len());
                        if let Err(err) = sender.send(repos).await {
                            error!("Failed to process repositories: {}", err);
                        }
                        page_num = page_num + 1;
                        repo_count = std::cmp::max(repo_count, MAX_REPOS_PAGE) - MAX_REPOS_PAGE;
                    }
                    Err(err) => {
                        error!("Failed to get repositories: {}", err);
                        break;
                    }
                }
            }
        });
        receiver
    }

    async fn top_repos_page(
        client: &Arc<CLIENT>,
        lang: impl Into<String>,
        repo_count: u32,
        page_num: u32,
    ) -> clients::api::Result<Vec<REPO>> {
        if repo_count < MAX_REPOS_PAGE {
            if page_num == FIRST_PAGE_NUMBER {
                client.top_repos(lang.into(), page_num, repo_count).await
            } else {
                let repos = client.top_repos(lang.into(), page_num, MAX_REPOS_PAGE).await;
                repos.map(|v| Self::take_first_n(v, repo_count))
            }
        } else {
            client.top_repos(lang.into(), page_num, MAX_REPOS_PAGE).await
        }
    }

    fn take_first_n<T>(v: Vec<T>, n: u32) -> Vec<T> {
        v.into_iter().take(n as usize).collect()
    }

    fn top_contributors(
        repo_receiver: Receiver<Vec<REPO>>,
        client: Arc<CLIENT>,
        threshold: f32,
    ) -> Receiver<BusFactor> {
        let (bus_factor_sender, bus_factor_receiver) = tokio::sync::mpsc::channel::<BusFactor>(10);
        tokio::spawn(async move {
            ReceiverStream::new(repo_receiver)
                .flat_map(|repos| stream::iter(repos))
                .map(|repo| Self::repo_bus_factor(repo, client.clone(), threshold))
                .for_each(|handle| async {
                    if let Ok(Some(p)) = handle.await {
                        if let Err(err) = bus_factor_sender.send(p).await {
                            error!("Failure: {}", err);
                        }
                    }
                })
                .await;
        });
        bus_factor_receiver
    }

    fn repo_bus_factor(repo: REPO, client: Arc<CLIENT>, threshold: f32) -> JoinHandle<Option<BusFactor>> {
        // TODO add parameter
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
