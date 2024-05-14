use std::sync::Arc;

use anyhow::{Context, Result};
use socks5_server::{Associate, Bind, Command, Connect, IncomingConnection};
use socks5_server::connection::associate::state::NeedReply as AssociateNeedReply;
use socks5_server::connection::bind::state::NeedFirstReply;
use socks5_server::connection::connect::state::NeedReply as ConnectNeedReply;
use socks5_server::connection::state::{NeedAuthenticate, NeedCommand};
use socks5_server::proto::{Address, Reply};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpSocket;
use tracing::{error, info, instrument};

use crate::config::Config;

#[instrument(skip(connect, config))]
pub async fn handle(connect: IncomingConnection<(), NeedAuthenticate>, config: Arc<Config>) {
    if let Err(e) = handle_impl(connect, config).await {
        error!("{}", e);
    }
}

async fn handle_impl(
    connect: IncomingConnection<(), NeedAuthenticate>,
    config: Arc<Config>,
) -> Result<()> {
    let conn: IncomingConnection<(), NeedCommand> = match connect.authenticate().await {
        Ok((conn, _)) => conn,
        Err((e, mut conn)) => {
            conn.shutdown().await?;
            Err(e).with_context(|| "Unable to authenticate")?
        }
    };
    info!("authenticated");

    let cmd = match conn.wait().await {
        Ok(cmd) => cmd,
        Err((e, mut conn)) => {
            conn.shutdown().await?;
            Err(e).with_context(|| "Unable to parse SOCKS command")?
        }
    };

    match cmd {
        Command::Associate(ass, addr) => {
            info!(%addr, "Associate");
            handle_associate(ass).await
        }
        Command::Bind(bind, addr) => {
            info!(%addr, "Bind");
            handle_bind(bind).await
        }
        Command::Connect(conn, addr) => {
            info!(%addr, "Connect");
            handle_connect(config, conn, addr).await
        }
    }
}

#[instrument(name = "connect", skip_all)]
async fn handle_connect(
    config: Arc<Config>,
    conn: Connect<ConnectNeedReply>,
    addr: Address,
) -> Result<()> {
    let target = match addr {
        Address::SocketAddress(a) => a,
        Address::DomainAddress(domain, port) => {
            let d = String::from_utf8_lossy(&domain);
            #[allow(clippy::let_and_return)] // Needed for d to live long enough
            let a = tokio::net::lookup_host((d.as_ref(), port))
                .await
                .with_context(|| format!("Unable to resolve {}", d))?
                .next()
                .with_context(|| format!("Resolving {} returned 0 results", d))?;
            a
        }
    };

    let socket = if target.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    socket
        .bind_device(Some(config.bind_interface.as_bytes()))
        .with_context(|| format!("Unable to bind to {}", config.bind_interface))?;
    match socket.connect(target).await {
        Ok(mut target_conn) => {
            info!(%target, "connected");
            let reply = conn
                .reply(Reply::Succeeded, Address::SocketAddress(target))
                .await;

            let mut reply_conn = match reply {
                Ok(conn) => conn,
                Err((err, mut conn)) => {
                    conn.shutdown().await?;
                    Err(err)?
                }
            };

            let res = tokio::io::copy_bidirectional(&mut target_conn, &mut reply_conn).await;
            reply_conn.shutdown().await?;
            target_conn.shutdown().await?;
            info!(%target, "disconnected");
            res?;
        }
        Err(_) => {
            let reply = conn
                .reply(Reply::HostUnreachable, Address::unspecified())
                .await;

            let mut conn = match reply {
                Ok(conn) => conn,
                Err((err, mut conn)) => {
                    conn.shutdown().await?;
                    Err(err)?
                }
            };
            conn.close().await?;
        }
    }
    Ok(())
}

#[instrument(name = "associate")]
async fn handle_associate(ass: Associate<AssociateNeedReply>) -> Result<()> {
    let reply = ass
        .reply(Reply::CommandNotSupported, Address::unspecified())
        .await;

    let mut conn = match reply {
        Ok(conn) => conn,
        Err((err, mut conn)) => {
            conn.shutdown().await?;
            Err(err)?
        }
    };
    conn.close().await?;
    Ok(())
}

#[instrument(name = "bind")]
async fn handle_bind(bind: Bind<NeedFirstReply>) -> Result<()> {
    let reply = bind
        .reply(Reply::CommandNotSupported, Address::unspecified())
        .await;

    let mut conn = match reply {
        Ok(conn) => conn,
        Err((err, mut conn)) => {
            conn.shutdown().await?;
            Err(err)?
        }
    };
    conn.close().await?;
    Ok(())
}
