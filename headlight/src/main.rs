use std::{net::TcpStream, thread::sleep, time::Duration};

use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals, rmt::{FixedLengthSignal, PinState, Pulse, TxRmtConfig, TxRmtDriver}};
use esp_idf_svc::{eventloop::EspSystemEventLoop, mdns::{EspMdns, Interface, Protocol, QueryResult, Type}, nvs::EspDefaultNvsPartition, wifi::{ClientConfiguration, Configuration, EspWifi}};
use esp_idf_sys::mdns_result_t;

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

    wifi.set_configuration(&Configuration::Client(ClientConfiguration{ ssid: ssid.try_into().unwrap(), password: password.try_into().unwrap(), ..Default::default() })).unwrap();
    
    wifi.start().unwrap();
    wifi.connect().unwrap();

    


    // let stream = TcpStream::connect(host_ip).unwrap();


    

    while !wifi.is_connected().unwrap(){
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::from_secs_f32(0.1));
    }
    println!("Should be connected now");

    println!("IP info: {:?}", wifi.sta_netif().get_ip_info().unwrap());


    let mdns = EspMdns::take().unwrap();
    let mut results = [QueryResult{
        instance_name: None,
        hostname: None,
        port: 0,
        txt: Vec::new(),
        addr: Vec::new(),
        interface: Interface::STA,
        ip_protocol: Protocol::V4, 
    }];

    let host_ip = loop{
        sleep(Duration::from_secs_f32(0.1));
        let result = mdns.query_srv("RtAudioEffect", "_RtAudioEffect", "_tcp", Duration::from_secs(5), &mut results);
        match result {
            Ok(count) => {
                if count == 0 {
                    println!("No results found");
                }
                else {
                    println!("Found {:} results: {:?}", count, results[0]);
                }
                continue;
            },
            Err(e) => {
                println!("Error: {:}", e);
            }
        }
    };


    let led_pin = peripherals.pins.gpio2;
    let channel = peripherals.rmt.channel0;
    let config = TxRmtConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, led_pin, &config).unwrap();
    // 3 seconds white at 10% brightness
    neopixel(Rgb::new(25, 25, 25), &mut tx).unwrap();
    FreeRtos::delay_ms(3000);




    // infinite rainbow loop at 100% brightness
    (0..360).cycle().try_for_each(|hue| {
        FreeRtos::delay_ms(10);
        let rgb = Rgb::from_hsv(hue, 100, 100);
        neopixel(rgb, &mut tx)
    }).unwrap()
}

fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<(), Box<dyn std::error::Error>> 
{
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
    b: u8
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