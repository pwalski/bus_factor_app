use bus_factor::api::Error;
use bus_factor_app::args::Args;
use clap::Parser;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let bus_factor_stream = bus_factor_app::calculate_bus_factor(args).await?;

    bus_factor_stream
        .for_each(|bus_factor| async move {
            println!("{}", bus_factor);
        })
        .await;

    Ok(())
}
