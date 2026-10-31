// src/bin/server.rs
use anyhow::Result;
use clap::Parser;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal;
use tracing::{error, info, warn};
use udp_tokio_stack::proto::Message;
use udp_tokio_stack::service::handle_message;
use udp_tokio_stack::{make_broadcast_socket, make_multicast_socket};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to bind unicast listener to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Bind address for unicast listener
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Broadcast port to listen on
    #[arg(long, default_value_t = 8081)]
    broadcast_port: u16,

    /// Multicast group to join
    #[arg(long, default_value = "239.255.42.42")]
    mcast_group: String,

    /// Multicast port
    #[arg(long, default_value_t = 8082)]
    mcast_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("Starting UDP Tokio Stack Server...");

    // Unicast
    let unicast_addr = format!("{}:{}", args.bind, args.port);
    let unicast = UdpSocket::bind(&unicast_addr).await?;
    info!("Unicast listener on {}", unicast_addr);

    // Broadcast
    let broadcast_addr = format!("0.0.0.0:{}", args.broadcast_port);
    let broadcast = make_broadcast_socket(&broadcast_addr)?;
    info!("Broadcast listener on {}", broadcast_addr);

    // Multicast
    let mcast_addr = format!("0.0.0.0:{}", args.mcast_port);
    let group: Ipv4Addr = args
        .mcast_group
        .parse()
        .expect("Invalid multicast group IPv4");
    let interface = Ipv4Addr::new(0, 0, 0, 0);
    let multicast = make_multicast_socket(&mcast_addr, group, interface)?;
    info!("Multicast listener on {} group {}", mcast_addr, group);

    // Spawn listeners
    let s1 = tokio::spawn(run_socket(unicast, "unicast".to_string()));
    let s2 = tokio::spawn(run_socket(broadcast, "broadcast".to_string()));
    let s3 = tokio::spawn(run_socket(multicast, "multicast".to_string()));

    // Graceful shutdown
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, exiting...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Abort tasks
    s1.abort();
    s2.abort();
    s3.abort();

    Ok(())
}

async fn run_socket(socket: UdpSocket, label: String) {
    let socket = Arc::new(socket);
    let mut buf = [0u8; 2048];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                let data = &buf[..len];
                match bincode::deserialize::<Message>(data) {
                    Ok(msg) => {
                        info!(%label, ?src, "Received message");
                        let sock = socket.clone();
                        tokio::spawn(async move {
                            if let Ok(reply) = handle_message(msg).await {
                                if let Ok(bytes) = bincode::serialize(&reply) {
                                    if let Err(e) = sock.send_to(&bytes, &src).await {
                                        error!("Failed to send reply to {}: {}", src, e);
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!(%label, ?src, "Failed to decode: {}", e);
                    }
                }
            }
            Err(e) => {
                error!(%label, "Recv error: {}", e);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}
