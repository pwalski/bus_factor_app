use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use bus_factor::{BusFactorCalculator, BusFactorStream};
use clap::Parser;
use clients::api::Result;
use github_client::GithubClientBuilder;
use secrecy::SecretString;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Repositories language
    #[clap(short, long)]
    pub language: String,

    /// Number of times to greet
    #[clap(short, long)]
    pub project_count: u32,

    /// API OAuth access token
    #[clap(long)]
    pub api_token: Option<SecretString>,

    /// Repository API URL
    #[clap(long, default_value = "https://api.github.com")]
    pub api_url: String,

    /// Bus factor threshold
    #[clap(long, default_value_t = 0.75, parse(try_from_str=threshold_in_range))]
    pub threshold: f32,

    /// Maximal parallel repository search requests
    #[clap(long, default_value_t = 1, parse(try_from_str=max_repo_req_in_range))]
    pub max_repo_req: u32,

    /// Maximal parallel repository contributors requests
    #[clap(long, default_value_t = 10, parse(try_from_str=max_contrib_req_in_range))]
    pub max_contrib_req: u32,
}

fn threshold_in_range(value: &str) -> clap::Result<f32, String> {
    //TODO min == 0.0 makes no sense but wanted to reuse method...
    number_in_range(value, 0.0, 1.0, "threshold".to_string())
}

fn max_repo_req_in_range(value: &str) -> clap::Result<u32, String> {
    //TODO min == 0.0 makes no sense but wanted to reuse method...
    number_in_range(value, 1, u32::MAX, "max_repo_req".to_string())
}

fn max_contrib_req_in_range(value: &str) -> clap::Result<u32, String> {
    //TODO min == 0.0 makes no sense but wanted to reuse method...
    number_in_range(value, 1, u32::MAX, "max_contrib_req".to_string())
}

fn number_in_range<T>(value: &str, min: T, max: T, name: String) -> clap::Result<T, String>
where
    T: FromStr + PartialOrd + Display,
    <T as FromStr>::Err: Display,
{
    value.parse::<T>().map_err(|err| format!("{}", err)).and_then(|value| {
        if value < min || value > max {
            return Err(format!("{} is not in range {} .. {}.", name, min, max));
        }
        Ok(value)
    })
}

pub fn calculate_bus_factor(args: Args) -> Result<BusFactorStream> {
    env_logger::init();

    let mut client = GithubClientBuilder::default().with_github_url(args.api_url);
    if let Some(token) = args.api_token {
        client = client.try_with_token(token)?;
    }
    let client = client.build()?;
    let calculator = BusFactorCalculator::new(client, args.threshold);
    Ok(calculator.calculate(
        args.language,
        args.project_count,
        args.max_repo_req as usize,
        args.max_contrib_req as usize,
    ))
}
