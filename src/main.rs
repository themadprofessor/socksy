//! Dirt simple SOCKS5 CONNECT proxy which binds a network interface

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::Parser;
use figment::Figment;
use figment::providers::{Env, Serialized};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use socks5_server::auth::NoAuth;
use socks5_server::Server;
use tokio::net::TcpListener;
use tracing::{error, info, instrument, warn};

use crate::config::{Args, Config};

mod config;
mod handler;

#[tokio::main]
#[instrument]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        error!("{:?}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config: Arc<Config> = Arc::new(
        Figment::new()
            .merge(Serialized::defaults(config::defaults()))
            .merge(Env::prefixed("SOCKSY_"))
            .merge(Serialized::defaults(Args::parse()))
            .extract()
            .with_context(|| "Invalid arguments or configuration")?,
    );

    match NetworkInterface::show() {
        Ok(inters) => {
            if !inters.iter().any(|i| i.name == config.bind_interface) {
                bail!("{} is not a known network interface", config.bind_interface);
            }
        }
        Err(e) => {
            warn!(
                "failed to fetch network interfaces for configuration validation, skipping. {}",
                e
            );
        }
    }

    let server = Server::new(
        TcpListener::bind(&config.listen_address).await?,
        Arc::new(NoAuth),
    );

    info!(listen_address = &config.listen_address, "server started");
    while let Ok((connect, addr)) = server.accept().await {
        info!(%addr, "connection received");
        let conf = config.clone();
        tokio::spawn(handler::handle(connect, conf));
    }

    Ok(())
}
