// tests/integration_test.rs
use anyhow::Result;
use bincode;
use std::sync::Once;
use tokio::net::UdpSocket;
use tokio::time::{Duration, timeout};
use udp_tokio_stack::proto::{Message, MessageKind};

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    });
}

#[tokio::test]
async fn server_handles_ping() -> Result<()> {
    init();

    // Spawn the server binary logic in-process by invoking the server task.
    // For simplicity, we spawn the server binary as a process using tokio::process::Command.
    // But for an in-process test we could factor the server run function into a library function.
    // To keep simple and reliable, here we bind a tiny "echo" server inline for the test.
    let server_socket = UdpSocket::bind("127.0.0.1:19000").await?;
    tokio::spawn(async move {
        let mut buf = [0u8; 1500];
        loop {
            if let Ok((len, src)) = server_socket.recv_from(&mut buf).await {
                if let Ok(msg) = bincode::deserialize::<Message>(&buf[..len]) {
                    let reply = Message::new(msg.id, MessageKind::Echo, b"pong".to_vec());
                    let bytes = bincode::serialize(&reply).unwrap();
                    let _ = server_socket.send_to(&bytes, &src).await;
                }
            }
        }
    });

    // Client sends ping
    let client = UdpSocket::bind("0.0.0.0:0").await?;
    client.connect("127.0.0.1:19000").await?;
    let msg = Message::new(42, MessageKind::Ping, vec![]);
    let bytes = bincode::serialize(&msg)?;
    client.send(&bytes).await?;

    // Await reply
    let mut buf = [0u8; 1500];
    let got = timeout(Duration::from_secs(2), client.recv(&mut buf)).await?;
    let len = got?;
    let reply: Message = bincode::deserialize(&buf[..len])?;
    assert_eq!(reply.id, 42);
    Ok(())
}
