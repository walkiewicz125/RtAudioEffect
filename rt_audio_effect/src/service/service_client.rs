use std::net::{SocketAddr, TcpStream};

use log::{debug, info};

use headlight_if::Message;
use serializer::ByteMessagePort;

pub struct ServiceClient {
    pub port: ByteMessagePort<Message>,
    pub addr: SocketAddr,
}

impl ServiceClient {
    pub fn new(connection: (TcpStream, SocketAddr)) -> Self {
        info!("Client connected from: {}", connection.1);

        Self {
            port: ByteMessagePort::new(connection.0),
            addr: connection.1,
        }
    }

    pub fn recv_message(&mut self) -> Result<Message, String> {
        let message = self.port.recv()?;
        debug!("Received message: {:?}", message);
        Ok(message)
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), String> {
        debug!("Sending message: {:?}", message);
        self.port.send(message)
    }
}
