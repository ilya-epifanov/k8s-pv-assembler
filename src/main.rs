use anyhow::Result;
use controller::db::Db;
use kube::Client;
use opts::Opts;
use std::net::SocketAddr;
use std::str::FromStr;

mod controller;
mod metrics;
mod opts;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::init_from_env()?;

    let addr = SocketAddr::from_str("0.0.0.0:80")?;

    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let db = Db::new(sled::open("/data/db")?);

    tokio::spawn(metrics::metrics(addr));
    controller::controller(kubernetes_client, &opts, db).await?;

    Ok(())
}
