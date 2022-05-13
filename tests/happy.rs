use bus_factor::BusFactor;
use bus_factor_app::calculate_bus_factor;
use bus_factor_app::Args;
use std::collections::VecDeque;
use tokio;
use wiremock::http::Method;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::Match;
use wiremock::Request;
use wiremock::{Mock, MockServer, ResponseTemplate};

const MAX_REPOS_PAGE: u32 = 100;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn happy_path_300() {
    let server = MockServer::start().await;

    const REPOS_COUNT: u32 = 1000;
    // Every Nth repo will have a large bus factor
    const BUS_FACTOR_DIVISOR: u32 = 5;
    const REPO_CONTRBRS_COUNT: u32 = 25;
    const LANG: &str = "rust";

    mock_repos(&server, REPOS_COUNT, LANG.to_string()).await;

    let mut bus_factors = mock_contributors(&server, REPOS_COUNT, REPO_CONTRBRS_COUNT, BUS_FACTOR_DIVISOR).await;

    let args = Args {
        language: LANG.to_string(),
        project_count: REPOS_COUNT,
        api_token: None,
        api_url: server.uri(),
        threshold: 0.75,
    };

    let mut receiver = calculate_bus_factor(args).await.unwrap();

    while let Some(factor) = receiver.recv().await {
        if let Some(expected_factor) = bus_factors.pop_front() {
            assert_eq!(factor, expected_factor);
        } else {
            panic!("Got unexpected result: {}", factor);
        }
    }
    assert!(
        bus_factors.is_empty(),
        "Every BUS_FACTOR_DIVISOR-th repo should has a bus factor over threshold and it should be reported"
    );
}

async fn mock_repos<'a>(server: &'a MockServer, repos_count: u32, lang: String) {
    for repo_page in 0..repos_count / MAX_REPOS_PAGE {
        let mut body = String::from(
            r#"{
                "total_count": 319021,
                "incomplete_results": false,
                "items": ["#,
        );
        for repo_page_index in 0..MAX_REPOS_PAGE {
            let repo_index = repo_page * MAX_REPOS_PAGE + repo_page_index;
            body.push_str(&format!(
                r#"{{
                    "name": "repo_{}",
                    "owner": {{
                        "login": "owner_{}"
                    }}
                }}"#,
                repo_index, repo_index
            ));
            middle_coma(&mut body, repo_page_index, MAX_REPOS_PAGE - 1);
        }
        body.push_str(
            r#"]
                }"#,
        );
        Mock::given(method("GET"))
            .and(path("/search/repositories"))
            .and(query_param("q", format!("language:{}", lang)))
            .and(query_param("sort", "stars"))
            .and(query_param("order", "desc"))
            .and(query_param("per_page", format!("{}", MAX_REPOS_PAGE)))
            .and(query_param("page", format!("{}", repo_page + 1)))
            .and(header("Accept", "application/vnd.github.v3+json"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(server)
            .await;
    }
}

async fn mock_contributors<'a>(
    server: &'a MockServer,
    repos_count: u32,
    repo_contributors_count: u32,
    bus_factor_divisor: u32,
) -> VecDeque<BusFactor> {
    let mut bus_factors = VecDeque::new();
    for repo_index in 0..repos_count {
        let mut user_contributions = 25;

        let mut body = String::from(r#"["#);
        // Every 5th repo will have bus factor (1st contributor with 1000 contributions)
        let login = contributor_login(repo_index, 0);
        let will_have_bus_factor = repo_index % bus_factor_divisor == 0;
        if will_have_bus_factor {
            body.push_str(&contribution_body(&login, 1000));
            bus_factors.push_back(BusFactor::new(format!("repo_{}", repo_index), login, 0.77));
        } else {
            body.push_str(&contribution_body(&login, user_contributions));
        };

        // Other contributors
        for repo_contributor_index in 1..repo_contributors_count {
            user_contributions = user_contributions - 1;
            middle_coma(&mut body, repo_contributor_index, repo_contributors_count);
            let login = contributor_login(repo_index, repo_contributor_index);
            body.push_str(&contribution_body(&login, user_contributions));
        }
        body.push_str(r#"]"#);

        //TODO Figure out why wiremock path matcher does not work.
        let p = format!("/repos/owner_{}/repo_{}/contributors", repo_index, repo_index);
        Mock::given(GetPathMatcher(p))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(server)
            .await;
    }
    return bus_factors;
}

//TODO Delete it and use proper path matcher
pub struct GetPathMatcher(String);
impl Match for GetPathMatcher {
    fn matches(&self, request: &Request) -> bool {
        request.method == Method::Get && request.url.path() == self.0
    }
}

fn contribution_body(login: &String, contributions: u32) -> String {
    format!(r#"{{ "login": "{}", "contributions": {} }}"#, login, contributions)
}

fn contributor_login(repo_index: u32, repo_contributor_index: u32) -> String {
    format!("login_{}_{}", repo_index, repo_contributor_index)
}

fn middle_coma(body: &mut String, index: u32, end: u32) {
    if index < end {
        body.push_str(",");
    }
}
