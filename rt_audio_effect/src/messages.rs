use log::{error, info};
use serializer::Serializable;

use crate::service::MessageFrame;

#[derive(Serializable, Default, Debug)]
pub struct EchoMessage {
    pub message: String,
}

#[derive(Serializable, Debug)]
pub enum Message {
    Invalid,
    Echo(EchoMessage),
}

impl Default for Message {
    fn default() -> Self {
        Message::Invalid
    }
}

impl Into<Message> for MessageFrame {
    fn into(self) -> Message {
        match self.id() {
            0 => {
                if let Ok(message) = EchoMessage::try_from_bytes(self.data()) {
                    info!("Received EchoMessage: {:?}", message);
                    return Message::Echo(message);
                } else {
                    error!("Failed to parse EchoMessage");
                    return Message::Invalid;
                }
            }
            _ => Message::Invalid,
        }
    }
}
