use bus_factor::{BusFactor, BusFactorCalculator};
use clap::Parser;
use clients::api::Result;
use github_client::GithubClientBuilder;
use secrecy::SecretString;
use tokio::sync::mpsc::Receiver;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Repositories language
    #[clap(short, long)]
    language: String,

    /// Number of times to greet
    #[clap(short, long)]
    project_count: u32,

    /// API OAuth access token
    #[clap(long)]
    api_token: Option<SecretString>,

    /// Repository API URL
    #[clap(long, default_value = "https://api.github.com")]
    api_url: String,

    /// Bus factor threshold
    #[clap(long, default_value_t = 0.75, parse(try_from_str=threshold_in_range))]
    threshold: f32,
}

fn threshold_in_range(threshold: &str) -> clap::Result<f32, String> {
    threshold
        .parse::<f32>()
        .map_err(|err| format!("{}", err))
        .and_then(|threshold| {
            if threshold <= 0.0 || threshold > 1.0 {
                return Err(format!("Threshold {} is not between (0.0, 1.0].", threshold));
            }
            Ok(threshold)
        })
}

pub async fn calculate_bus_factor(args: Args) -> Result<Receiver<BusFactor>> {
    env_logger::init();

    let mut client = GithubClientBuilder::default().with_github_url(args.api_url);
    if let Some(token) = args.api_token {
        client = client.try_with_token(token)?;
    }
    let client = client.build()?;

    let calculator = BusFactorCalculator::new(client, args.threshold);
    let receiver = calculator.calculate(args.language, args.project_count);

    Ok(receiver)
}
