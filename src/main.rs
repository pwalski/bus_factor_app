use bus_factor::BusFactor;
use clients::api::Error;
use github_client::GithubClientBuilder;
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let bus_factor = GithubClientBuilder::default().build().map(BusFactor::new)?;
    let mut receiver = bus_factor.calculate("rust", 10);
    while let Some(res) = receiver.recv().await {
        println!("{}", res);
    }
    Ok(())
}
