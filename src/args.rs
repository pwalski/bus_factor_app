use bus_factor::api::Sort;
use clap::Parser;
use secrecy::SecretString;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Repositories language
    #[clap(short, long, env)]
    pub language: String,

    /// Number of times to greet
    #[clap(short, long, env)]
    pub project_count: u32,

    #[clap(short, long, env, default_value = "stars")]
    pub sort: Sort,

    /// API OAuth access token
    #[clap(short, long, env)]
    pub api_token: Option<SecretString>,

    /// Repository API URL
    #[clap(long, env, default_value = "https://api.github.com")]
    pub api_url: String,

    /// Bus factor threshold
    #[clap(short, long, env, default_value_t = 0.75, parse(try_from_str=threshold_in_range))]
    pub threshold: f32,

    /// Maximal parallel repository search requests
    #[clap(long, env, default_value_t = 1, parse(try_from_str=max_repo_req_in_range))]
    pub max_repo_req: u32,

    /// Maximal parallel repository contributors requests
    #[clap(long, env, default_value_t = 10, parse(try_from_str=max_contrib_req_in_range))]
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
