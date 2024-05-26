use std::net::TcpListener;

use log::info;
use mdns_sd::ServiceInfo;

use super::ServiceClient;

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
