// src/bin/client.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::net::UdpSocket;
use tracing::{error, info, warn};
use udp_tokio_stack::make_broadcast_socket;
use udp_tokio_stack::proto::{Message, MessageKind};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send unicast ping
    Unicast {
        #[arg(long, default_value = "127.0.0.1:8080")]
        target: String,
    },
    /// Send broadcast message
    Broadcast {
        #[arg(long, default_value = "255.255.255.255:8081")]
        target: String,
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: String,
    },
    /// Send multicast message
    Multicast {
        #[arg(long, default_value = "239.255.42.42:8082")]
        target: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.command {
        Commands::Unicast { target } => {
            info!("Running Unicast example -> {}", target);
            unicast_example(&target).await?;
        }
        Commands::Broadcast { target, bind } => {
            info!(
                "Running Broadcast example -> {} (bound to {})",
                target, bind
            );
            broadcast_example(&target, &bind).await?;
        }
        Commands::Multicast { target } => {
            info!("Running Multicast example -> {}", target);
            multicast_example(&target).await?;
        }
    }

    Ok(())
}

async fn unicast_example(target: &str) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(target).await?;

    let msg = Message::new(1, MessageKind::Ping, vec![]);
    let bytes = bincode::serialize(&msg)?;
    socket.send(&bytes).await?;
    info!("Sent Ping to unicast server");

    let mut buf = [0u8; 1500];
    let len = socket.recv(&mut buf).await?;
    let reply: Message = bincode::deserialize(&buf[..len])?;
    info!("Unicast reply: {:?}", reply);
    Ok(())
}

async fn broadcast_example(target: &str, bind: &str) -> Result<()> {
    // Use std socket wrapper that has broadcast set
    let socket = make_broadcast_socket(bind)?;
    // send to broadcast address
    let msg = Message::new(2, MessageKind::Broadcast, b"hello broadcast".to_vec());
    let bytes = bincode::serialize(&msg)?;
    socket.send_to(&bytes, target).await?;
    info!("Sent broadcast to {}", target);

    // Try to receive (optional) with timeout
    let mut buf = [0u8; 1500];
    match tokio::time::timeout(
        std::time::Duration::from_secs(1),
        socket.recv_from(&mut buf),
    )
    .await
    {
        Ok(Ok((len, src))) => {
            // Try to deserialize if we got something
            if let Ok(reply) = bincode::deserialize::<Message>(&buf[..len]) {
                info!("Got broadcast reply from {}: {:?}", src, reply);
            } else {
                warn!("Got data from {} but failed to deserialize", src);
            }
        }
        Ok(Err(e)) => error!("Recv error: {}", e),
        Err(_) => info!("Timeout waiting for broadcast reply (this is expected if no peers)"),
    }
    Ok(())
}

async fn multicast_example(group: &str) -> Result<()> {
    // For sending multicast, we can just use a normal UDP socket if we are just sending TO the group.
    // If we want to receive, we need membership. Client here just sends.

    // We might need to set TTL if we want it to go further, but for local tests default (1) is usually fine.
    // To be safe/professional let's use the helper if we were receiving, but here we just send.
    // But setting TTL on a Tokio socket requires accessing the std socket or using socket2 (too complex for this toy refactor).
    // Let's assume default TTL is fine for now, or just use `socket.set_ttl` if available (it's not on tokio::UdpSocket directly without socket2/libc usually).
    // Actually, to set TTL we can use `udp_from_std`.

    let std_sock = std::net::UdpSocket::bind("0.0.0.0:0")?;
    std_sock.set_multicast_ttl_v4(1)?;
    let socket = tokio::net::UdpSocket::from_std(std_sock)?;

    let msg = Message::new(3, MessageKind::Multicast, b"hello mcast".to_vec());
    let bytes = bincode::serialize(&msg)?;
    socket.send_to(&bytes, group).await?;
    info!("Sent multicast to {}", group);
    Ok(())
}
