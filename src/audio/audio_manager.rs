use cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
    Device,
};

use log::{debug, info};

pub struct AudioManager {}

impl AudioManager {
    pub fn get_default_loopback() -> Result<Device, &'static str> {
        let available_hosts = cpal::available_hosts();
        info!("Searching for default loopback device");

        debug!("Available hosts:");
        for host in &available_hosts {
            debug!("Host name: {host:#?}");
        }

        if available_hosts.is_empty() {
            return Err("No host devices available found");
        }
        let selected_host_id = available_hosts[0];

        info!("Selected audio host: {selected_host_id:#?}");

        let Ok(host) = cpal::host_from_id(selected_host_id) else {
            return Err("Failed to find Host");
        };

        let Some(device) = host.default_output_device() else {
            return Err("Failed to find output device");
        };

        debug!("Available audio output devices:");
        for output_device in host.output_devices().unwrap() {
            debug!("Audio device: {}", output_device.name().unwrap());
        }

        info!("Selected audio device: {}", device.name().unwrap());

        Ok(device)
    }
}
