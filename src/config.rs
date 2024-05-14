use clap::Parser;
use figment::map;
use figment::value::{Dict, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub listen_address: String,
    pub bind_interface: String,
}

#[derive(Parser, Debug, Deserialize, Serialize)]
#[command(version, author, about)]
/// Dirt simple SOCKS5 CONNECT proxy which binds a network interface
///
/// All command line arguments can also be provided as environment variables in SCREAMING_SNAKE_CASE
/// and prepended with `SOCKSY_`.
/// If both are used, the command line arguments take precedence.
///
/// For example: `SOCKSY_LISTEN_ADDRESS=127.0.0.1:8888` is the same as `--listen-address=127.0.0.1:8888`
pub(crate) struct Args {
    /// Address to listen for incoming connections on.
    ///
    /// Default: 127.0.0.1:8888
    #[arg(short, long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub listen_address: Option<String>,

    /// Name of the interface to bind to for outgoing connections.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub bind_interface: Option<String>,
}

pub(crate) fn defaults() -> Dict {
    map! {
        "listen_address".to_string() => Value::from("127.0.0.1:8888")
    }
}
