use bus_factor_app::Args;
use clap::Parser;
use clients::api::Error;

/// Simple program to greet a person

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mut receiver = bus_factor_app::calculate_bus_factor(args).await?;

    while let Some(res) = receiver.recv().await {
        println!("{}", res);
    }
    Ok(())
}
