use std::{
    f32::consts::E,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use log::{error, info};

use super::{Message, MessageFrame};

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
        }

        let mut len_buff: [u8; 4] = [0; 4];
        if let Err(err) = self.stream.read_exact(&mut len_buff) {
            error!("Failed to read message length: {:?}", err);
            return Message::Invalid;
        }

        let len = u32::from_le_bytes(len_buff);
        let mut data_buff = vec![0; len as usize];
        if let Err(err) = self.stream.read_exact(&mut data_buff) {
            error!("Failed to read message data");
            return Message::Invalid;
        };

        MessageFrame::new(u32::from_le_bytes(id_buff), data_buff).into()
    }

    pub fn send_message(&mut self, message: Message) {
        self.stream
            .write_all(&MessageFrame::from(message).get_bytes())
            .expect("Failed to send message");
    }
}
