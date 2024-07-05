use std::{default, sync::Mutex};

use log::{error, info};
use serializer::Serializable;

#[derive(Serializable, Default, Debug)]
pub enum Dupa {
    #[default]
    Asd,
    SSD(String),
}

#[derive(Serializable, Default, Debug)]
pub struct EchoMessage {
    pub message: String,
}

#[derive(Serializable, Default, Debug)]
pub struct IdentityMessage {
    pub info: String,
    pub effect_id: u8,
}

impl IdentityMessage {
    const INVALID: u8 = 0;
    const BASS: u8 = 1;
}

#[derive(Serializable, Default, Debug)]
pub struct IdentityRequestMessage {}

#[derive(Serializable, Default, Debug)]
pub struct SetColorMessage {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub enum Message {
    Invalid,
    Ack,
    Echo(EchoMessage),
    EchoReply(EchoMessage),
    IdentityRequest(IdentityRequestMessage),
    Identity(IdentityMessage),
    SetColor(SetColorMessage),
}

impl Into<Message> for MessageFrame {
    fn into(self) -> Message {
        match self.id() {
            0 => {
                error!("Received Invalid message");
                return Message::Invalid;
            }
            1 => {
                return Message::Ack;
            }
            2 => {
                if let Ok(message) = EchoMessage::try_from_bytes(self.data()) {
                    return Message::Echo(message);
                } else {
                    error!("Failed to parse Echo");
                    return Message::Invalid;
                }
            }
            3 => {
                if let Ok(message) = EchoMessage::try_from_bytes(self.data()) {
                    return Message::EchoReply(message);
                } else {
                    error!("Failed to parse EchoReply");
                    return Message::Invalid;
                }
            }
            4 => {
                if let Ok(message) = IdentityRequestMessage::try_from_bytes(self.data()) {
                    return Message::IdentityRequest(message);
                } else {
                    error!("Failed to parse IdentityRequest");
                    return Message::Invalid;
                }
            }
            5 => {
                if let Ok(message) = IdentityMessage::try_from_bytes(self.data()) {
                    return Message::Identity(message);
                } else {
                    error!("Failed to parse Identity");
                    return Message::Invalid;
                }
            }
            6 => {
                if let Ok(message) = SetColorMessage::try_from_bytes(self.data()) {
                    return Message::SetColor(message);
                } else {
                    error!("Failed to parse SetColor");
                    return Message::Invalid;
                }
            }
            _ => {
                error!("Received unknown message");
                return Message::Invalid;
            }
        }
    }
}

impl From<Message> for MessageFrame {
    fn from(message: Message) -> Self {
        match message {
            Message::Invalid => {
                error!("Trying to serialize invalid message");
                MessageFrame::new(0, Vec::new())
            }
            Message::Ack => MessageFrame::new(1, Vec::new()),
            Message::Echo(echo_message) => {
                let data = echo_message.get_bytes();
                MessageFrame::new(2, data)
            }
            Message::EchoReply(echo_message) => {
                let data = echo_message.get_bytes();
                MessageFrame::new(3, data)
            }
            Message::IdentityRequest(identity_request_message) => {
                let data = identity_request_message.get_bytes();
                MessageFrame::new(4, data)
            }
            Message::Identity(identity_message) => {
                let data = identity_message.get_bytes();
                MessageFrame::new(5, data)
            }
            Message::SetColor(set_color_message) => {
                let data = set_color_message.get_bytes();
                MessageFrame::new(6, data)
            }
        }
    }
}

#[derive(Debug)]
pub struct MessageFrame {
    id: u32,
    data: Vec<u8>,
}

impl MessageFrame {
    pub fn new(id: u32, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut bytes = self.id.to_le_bytes().to_vec();
        bytes.extend_from_slice(&((self.data.len() as u32).to_le_bytes()));
        bytes.extend_from_slice(&self.data);
        bytes
    }
}
