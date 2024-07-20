use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use log::{debug, error, info};

use headlight_if::Message;
use serializer::Serializable;

pub struct ServiceClient {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}

impl ServiceClient {
    pub fn new(connection: (TcpStream, SocketAddr)) -> Self {
        info!("Client connected from: {}", connection.1);

        Self {
            stream: connection.0,
            addr: connection.1,
        }
    }

    pub fn recv_message(&mut self) -> Message {
        let mut id_buff: [u8; 4] = [0; 4];
        if let Err(err) = self.stream.read_exact(&mut id_buff) {
            error!("Failed to read message id: {:?}", err);
            return Message::Invalid;
        } else {
            debug!("Received message id: {:?}", u32::from_le_bytes(id_buff));
        }

        let mut len_buff: [u8; 4] = [0; 4];
        if let Err(err) = self.stream.read_exact(&mut len_buff) {
            error!("Failed to read message length: {:?}", err);
            return Message::Invalid;
        } else {
            debug!(
                "Received message length: {:?}",
                u32::from_le_bytes(len_buff)
            );
        }

        let len = u32::from_le_bytes(len_buff);
        let mut data_buff = vec![0; len as usize];
        if let Err(err) = self.stream.read_exact(&mut data_buff) {
            error!("Failed to read message data: {:?}", err);
            return Message::Invalid;
        } else {
            debug!("Received message data: {:?}", data_buff);
        }

        match Message::try_from(data_buff) {
            Ok(message) => return message,
            Err(err) => {
                error!("Failed to parse message: {:?}", err);
                return Message::Invalid;
            }
        }
    }

    pub fn send_message(&mut self, message: Message) {
        debug!("Sending message: {:?}", message);

        let bytes: Vec<u8> = message.into();
        self.stream
            .write_all(bytes.as_slice())
            .expect("Failed to send message");
    }
}
