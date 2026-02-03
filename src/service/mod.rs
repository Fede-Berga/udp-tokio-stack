use crate::proto::{Message, MessageKind};
use crate::error::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, instrument};

#[instrument]
pub async fn handle_message(message: Message) -> Result<Message> {
    match message.kind {
        MessageKind::Ping => {
            info!("Handling Ping from {}", message.id);
            Ok(Message::new(message.id, MessageKind::Echo, b"Pong".to_vec()))
        }
        MessageKind::Echo => {
            info!("Handling Echo from {}", message.id);
            sleep(Duration::from_secs(1)).await;
            Ok(Message::new(message.id, MessageKind::Echo, message.data))
        }
        MessageKind::Broadcast => {
            info!("Handling Broadcast from {}", message.id);
            Ok(Message::new(message.id, MessageKind::Broadcast, b"ack".to_vec()))
        }
        MessageKind::Multicast => {
             info!("Handling Multicast from {}", message.id);
             Ok(Message::new(message.id, MessageKind::Multicast, b"ack".to_vec()))
        }
    }
}