use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{sleep, JoinHandle},
    time::{Duration, Instant},
};

use log::{error, info};
use mdns_sd::ServiceInfo;

use super::ServiceClient;

use headlight_if::{Message, SetColorMessage, SetServo};

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
    rainbow_thread: Option<JoinHandle<()>>,
    servo_thread: Option<JoinHandle<()>>,
}

pub enum ServoId {
    Servo1,
    Servo2,
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
            rainbow_thread: None,
            servo_thread: None,
        }
    }

    fn handle_message(message: Message, last_client: &mut ServiceClient) {
        match message {
            Message::Echo(msg) => {
                info!("Received Echo: {:?}", msg);
                if let Err(err) = last_client.send_message(Message::EchoReply(msg)) {
                    error!("Failed to send EchoReply: {}", err);
                } else {
                    info!("Sent EchoReply");
                }
            }
            _ => {}
        }
    }

    fn service_thread(shared_ctx: Arc<Mutex<ServiceSharedCtx>>) {
        loop {
            if let Ok(context_lock) = &mut shared_ctx.lock() {
                if !context_lock.is_alive {
                    break;
                }

                if let Ok((stream, address)) = context_lock.listner.accept() {
                    println!("New connection from: {}", address);
                    context_lock
                        .clients
                        .push(ServiceClient::new((stream, address)));
                }
            }

            sleep(Duration::from_millis(100));
        }
    }

    pub fn set_servo(&mut self, servo_id: ServoId, position: f32) {
        // generate sin wave
        let id = match servo_id {
            ServoId::Servo1 => 0,
            ServoId::Servo2 => 1,
        };

        if let Ok(context_lock) = &mut self.shared_ctx.try_lock() {
            for client in context_lock.clients.iter_mut() {
                client
                    .send_message(Message::SetServo(SetServo { id, position }))
                    .unwrap();
            }
        }
    }

    pub fn start(&mut self) {
        if let None = self.listner_thread {
            let shared_ctx = self.shared_ctx.clone();
            self.listner_thread = Some(std::thread::spawn(move || {
                Self::service_thread(shared_ctx);
            }));
        }
    }

    pub fn stop(&mut self) {
        self.shared_ctx.lock().unwrap().is_alive = false;
        self.listner_thread.take().map(|t| t.join().unwrap());
    }
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    /// Converts hue, saturation, value to RGB
    pub fn from_hsv(h: u32, s: u32, v: u32) -> Self {
        if h > 360 || s > 100 || v > 100 {
            panic!("The given HSV values are not in valid range");
        }
        let s = s as f64 / 100.0;
        let v = v as f64 / 100.0;
        let c = s * v;
        let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
        }
    }
}
