use std::{net::TcpStream, thread::sleep, time::Duration};
mod servo;
use esp_idf_hal::{
    delay::FreeRtos,
    ledc::{self, config::TimerConfig},
    peripherals::Peripherals,
    rmt::{FixedLengthSignal, PinState, Pulse, TxRmtConfig, TxRmtDriver},
    units::Hertz,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mdns::{EspMdns, Interface, Protocol, QueryResult},
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use headlight_if::{EchoMessage, IdentityMessage, IdentityRequestMessage, Message};
use serializer::ByteMessagePort;
use servo::*;

extern crate headlight_if;
extern crate serializer;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    let ssid = "D7D433";
    let password = "8RaycXmTcHcG";

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: password.try_into().unwrap(),
        ..Default::default()
    }))
    .unwrap();

    wifi.start().unwrap();
    wifi.connect().unwrap();

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::from_secs_f32(0.1));
    }
    println!("Should be connected now");

    println!("IP info: {:?}", wifi.sta_netif().get_ip_info().unwrap());

    let mdns = EspMdns::take().unwrap();
    let mut results = [QueryResult {
        instance_name: None,
        hostname: None,
        port: 0,
        txt: Vec::new(),
        addr: Vec::new(),
        interface: Interface::STA,
        ip_protocol: Protocol::V4,
    }];

    loop {
        sleep(Duration::from_secs_f32(0.1));
        let result = mdns.query_srv(
            "RtAudioEffect",
            "_RtAudioEffect",
            "_tcp",
            Duration::from_secs(5),
            &mut results,
        );
        match result {
            Ok(count) => {
                if count == 0 {
                    println!("No results found");
                } else {
                    println!("Found {:} results: {:?}", count, results[0]);
                }
                break;
            }
            Err(e) => {
                println!("Error: {:}", e);
            }
        }
    }

    let led_pin = peripherals.pins.gpio2;
    let channel = peripherals.rmt.channel0;
    let config = TxRmtConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, led_pin, &config).unwrap();
    // 3 seconds white at 10% brightness
    neopixel(Rgb::new(25, 25, 25), &mut tx).unwrap();
    FreeRtos::delay_ms(3000);

    let ip = results[0].addr.last().unwrap();
    let port = results[0].port;

    let stream = TcpStream::connect((*ip, port)).unwrap();

    let mut handler = CommandHandler::new(stream);

    let timer_driver = ledc::LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default()
            .frequency(Hertz(50))
            .resolution(ledc::Resolution::Bits14),
    )
    .unwrap();

    let mut servo1 = Servo::new(
        &timer_driver,
        peripherals.ledc.channel0,
        peripherals.pins.gpio7,
    )
    .unwrap();

    servo1.set_duty(0.5).unwrap();

    loop {
        handler.handle(|message| {
            println!("Command callback: {:?}", message);

            match message {
                Message::SetColor(set_color) => {
                    let rgb = Rgb::new(set_color.r, set_color.g, set_color.b);
                    neopixel(rgb, &mut tx).unwrap();
                }
                Message::SetServo(set_servo) => {
                    let duty = set_servo.position;
                    servo1.set_duty(duty).unwrap();
                }
                _ => {
                    println!("Unhandled message: {:?}", message);
                }
            }

            Some(())
        });
    }
}

struct CommandHandler {
    message_port: ByteMessagePort<Message>,
}

impl CommandHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            message_port: ByteMessagePort::new(stream),
        }
    }

    fn recv_message(&mut self) -> Message {
        match self.message_port.recv() {
            Ok(message) => {
                println!("Received message: {:?}", message);
                return message;
            }
            Err(err) => {
                println!("Failed to receive message: {:?}", err);
                return Message::Invalid;
            }
        }
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), String> {
        println!("Sending message: {:?}", message);
        if let Err(err) = self.message_port.send(message) {
            println!("Failed to send message: {:?}", err);
            return Err(err.to_string());
        }

        Ok(())
    }

    pub fn handle<F>(&mut self, mut callback: F) -> Option<()>
    where
        F: FnMut(&Message) -> Option<()>,
    {
        let message = self.recv_message();
        match &message {
            Message::Invalid => {
                println!("Invalid message");
                return callback(&message);
            }

            Message::Ack => {
                println!("Ack not implemented");
                return callback(&message);
            }

            Message::Echo(echo_message) => match self.handle_echo(&echo_message) {
                Ok(_) => {
                    println!("Echo handled");
                    return callback(&message);
                }
                Err(err) => {
                    println!("Failed to handle echo: {:?}", err);
                    return None;
                }
            },

            Message::IdentityRequest(request) => match self.handle_identity_request(&request) {
                Ok(_) => {
                    println!("Identity handled");
                    return callback(&message);
                }
                Err(err) => {
                    println!("Failed to handle identity: {:?}", err);
                    return None;
                }
            },

            _ => {
                println!(
                    "Unhandled message by handler: {:?}. Calling callback",
                    message
                );
                return callback(&message);
            }
        };
    }

    fn handle_echo(&mut self, echo_message: &EchoMessage) -> Result<(), String> {
        println!("Echo: {:?}", echo_message.message);

        let reply = Message::EchoReply(EchoMessage {
            message: format!("Reply: {}", echo_message.message),
        });

        self.send_message(reply)
    }

    fn handle_identity_request(&mut self, request: &IdentityRequestMessage) -> Result<(), String> {
        let _ = request;
        println!("Identity Request");

        let identity = Message::Identity(IdentityMessage {
            info: "ESP32".to_string(),
            effect_id: 0,
        });

        self.send_message(identity)
    }
}

fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<(), Box<dyn std::error::Error>> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = p & color != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    tx.start_blocking(&signal)?;
    Ok(())
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

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G        R        B
    /// 7      0 7      0 7      0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32
    }
}
