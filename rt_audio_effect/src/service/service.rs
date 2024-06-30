use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{sleep, JoinHandle},
    time::Duration,
};

use log::info;
use mdns_sd::ServiceInfo;

use super::ServiceClient;

use headlight_if::Message;

struct ServiceSharedCtx {
    is_alive: bool,
    listner: TcpListener,
    clients: Vec<ServiceClient>,
}

pub struct AudioHeadlightService {
    pub name: String,
    pub service_info: ServiceInfo,
    shared_ctx: Arc<Mutex<ServiceSharedCtx>>,
    listner_thread: Option<JoinHandle<()>>,
}

impl AudioHeadlightService {
    pub fn new(name: &str, listner: TcpListener, service_info: ServiceInfo) -> Self {
        info!("Service {} created", name);
        listner
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode");

        Self {
            name: name.to_string(),
            service_info,
            shared_ctx: Arc::new(Mutex::new(ServiceSharedCtx {
                is_alive: true,
                listner,
                clients: Vec::new(),
            })),
            listner_thread: None,
        }
    }

    fn listning_thread(shared_ctx: Arc<Mutex<ServiceSharedCtx>>) {
        while shared_ctx.lock().unwrap().is_alive {
            let conn = shared_ctx.lock().unwrap().listner.accept();
            match conn {
                Ok((stream, addr)) => {
                    info!("Client connected from: {}", addr);
                    let mut shared_ctx = shared_ctx.lock().unwrap();
                    shared_ctx.clients.push(ServiceClient::new((stream, addr)));

                    let last_client = shared_ctx.clients.last_mut().unwrap();
                    let msg = last_client.recv_message();
                    match msg {
                        Message::Echo(msg) => {
                            info!("Received Echo: {:?}", msg);
                            last_client.send_message(Message::EchoReply(msg));
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    sleep(Duration::from_millis(100));
                }
            }
        }
    }

    pub fn start(&mut self) {
        if let None = self.listner_thread {
            let shared_ctx = self.shared_ctx.clone();
            self.listner_thread = Some(std::thread::spawn(move || {
                Self::listning_thread(shared_ctx);
            }));
        }
    }

    pub fn stop(&mut self) {
        self.shared_ctx.lock().unwrap().is_alive = false;
        self.listner_thread.take().map(|t| t.join().unwrap());
    }
}
