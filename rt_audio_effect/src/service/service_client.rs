use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use log::info;

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
        self.stream
            .read_exact(&mut id_buff)
            .expect("Failed to read message id");

        let mut len_buff: [u8; 4] = [0; 4];
        self.stream
            .read_exact(&mut len_buff)
            .expect("Failed to read message length");

        let len = u32::from_le_bytes(len_buff);
        let mut data_buff = vec![0; len as usize];
        self.stream
            .read_exact(&mut data_buff)
            .expect("Failed to read message data");

        MessageFrame::new(u32::from_le_bytes(id_buff), data_buff).into()
    }

    pub fn send_message(&mut self, message: Message) {
        self.stream
            .write_all(&MessageFrame::from(message).get_bytes())
            .expect("Failed to send message");
    }
}
