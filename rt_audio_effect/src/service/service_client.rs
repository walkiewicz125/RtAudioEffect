use std::{
    io::Read,
    net::{SocketAddr, TcpStream},
};

use log::info;

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
}

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

    pub fn recv_message(&mut self) -> MessageFrame {
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

        MessageFrame::new(u32::from_le_bytes(id_buff), data_buff)
    }
}
