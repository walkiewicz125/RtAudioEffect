use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};

use log::info;
use mdns_sd::{ServiceDaemon, ServiceInfo};

pub struct ServiceClient {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}

impl ServiceClient {
    fn new(connection: (TcpStream, SocketAddr)) -> Self {
        info!("Client connected from: {}", connection.1);
        Self {
            stream: connection.0,
            addr: connection.1,
        }
    }
}

pub struct Service {
    pub name: String,
    pub listner: TcpListener,
    pub service_info: ServiceInfo,
}

impl Service {
    pub fn new(name: &str, listner: TcpListener, service_info: ServiceInfo) -> Self {
        info!("Service {} created", name);
        Self {
            name: name.to_string(),
            listner,
            service_info,
        }
    }

    pub fn wait_for_client(&self) -> ServiceClient {
        info!("Waiting for client connection");
        ServiceClient::new(
            self.listner
                .accept()
                .expect("Failed to accept client connection"),
        )
    }
}

pub struct ServiceRegister {
    mdns: ServiceDaemon,
    registered_services: Vec<Arc<Service>>,
}

impl ServiceRegister {
    pub fn new() -> Self {
        let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");

        Self {
            mdns,
            registered_services: Vec::new(),
        }
    }

    pub fn add_service(&mut self, name: &str) -> Arc<Service> {
        let listner = TcpListener::bind("0.0.0.0:0").expect("Failed to bind to random port");
        let port = listner.local_addr().expect("").port();
        info!("{} is listening on {}:{}", name, "localhost", port);

        let service_info = ServiceInfo::new(
            &format!("_{}._tcp.local.", name),
            name,
            &format!("{}.local.", name),
            "0.0.0.0",
            port,
            None,
        )
        .unwrap()
        .enable_addr_auto();

        self.mdns
            .register(service_info.clone())
            .expect("Failed to register RtAudioEffect service in mDNS deamon");

        let new_serivce = Arc::new(Service::new(name, listner, service_info));
        self.registered_services.push(new_serivce.clone());
        new_serivce
    }
}
