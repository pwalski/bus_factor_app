//TODO it is pub only for functional test...
pub mod args;

use args::Args;
use bus_factor::api::Result;
use bus_factor::{BusFactorCalculator, BusFactorStream};
use github_client::GithubClientBuilder;

pub async fn calculate_bus_factor(args: Args) -> Result<BusFactorStream> {
    env_logger::init();

    let mut client = GithubClientBuilder::default().with_github_url(args.api_url);
    if let Some(token) = args.api_token {
        client = client.try_with_token(token)?;
    }
    let client = client.build().await?;
    let calculator = BusFactorCalculator::new(client, args.threshold);
    Ok(calculator.calculate(
        args.language,
        args.project_count,
        args.max_repo_req as usize,
        args.max_contrib_req as usize,
    ))
}
