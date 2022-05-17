use bus_factor::{api::Error, BusFactor};
use bus_factor_app::args::Args;
use clap::Parser;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    bus_factor_app::calculate_bus_factor(args)
        .await?
        .for_each(print_line)
        .await;

    Ok(())
}
//TODO only because of for_each
async fn print_line(bus_factor: BusFactor) {
    let line = format!(
        "project: {0: <15} user: {1: <20} percentage: {2}",
        bus_factor.repo, bus_factor.contributor, bus_factor.percentage
    );
    println!("{}", line);
}
