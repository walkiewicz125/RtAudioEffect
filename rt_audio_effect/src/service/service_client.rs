use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use log::{debug, error, info};

use headlight_if::Message;
use headlight_if::MessageFrame;

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
            error!("Failed to read message data");
            return Message::Invalid;
        } else {
            debug!("Received message data: {:?}", data_buff);
        }

        MessageFrame::new(u32::from_le_bytes(id_buff), data_buff).into()
    }

    pub fn send_message(&mut self, message: Message) {
        let bytes = &MessageFrame::from(message).get_bytes();
        debug!("Sending message: {:?}", bytes);
        self.stream
            .write_all(bytes)
            .expect("Failed to send message");
    }
}
