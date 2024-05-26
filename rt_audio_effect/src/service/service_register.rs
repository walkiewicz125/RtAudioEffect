use std::{net::TcpListener, sync::Arc};

use log::info;
use mdns_sd::{ServiceDaemon, ServiceInfo};

use super::Service;

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
