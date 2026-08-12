use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};

pub fn get_available_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let mut device_names = Vec::new();

    // Add Output Devices (System Audio)
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                let display_name = format!("[Output] {}", name);
                if !device_names.contains(&display_name) {
                    device_names.push(display_name);
                }
            }
        }
    }

    // Add Input Devices (Microphones)
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                let display_name = format!("[Input] {}", name);
                if !device_names.contains(&display_name) {
                    device_names.push(display_name);
                }
            }
        }
    }

    Ok(device_names)
}

pub fn get_device_by_name(target_name: &str) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig)> {
    let host = cpal::default_host();
    
    // Check if it's an output device
    if target_name.starts_with("[Output]") {
        let clean_name = target_name.trim_start_matches("[Output] ");
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name == clean_name {
                        if let Ok(config) = device.default_output_config() {
                            return Ok((host, device, config));
                        }
                    }
                }
            }
        }
    }
    
    // Check if it's an input device
    if target_name.starts_with("[Input]") {
        let clean_name = target_name.trim_start_matches("[Input] ");
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name == clean_name {
                        if let Ok(config) = device.default_input_config() {
                            return Ok((host, device, config));
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Device not found or does not support default configuration"))
}

pub fn get_default_loopback_device() -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig)> {
    let host = cpal::default_host();

    // 1. Utamakan default output device (Speakers/Headphones) untuk WASAPI Loopback (Audio Sistem Windows)
    if let Some(device) = host.default_output_device() {
        if let Ok(config) = device.default_output_config() {
            return Ok((host, device, config));
        }
    }

    // 2. Fallback ke input device (Mikrofon) jika output device tidak tersedia
    let device = host
        .default_input_device()
        .context("Tidak dapat menemukan perangkat audio (output/input) default pada OS")?;

    let config = device
        .default_input_config()
        .context("Gagal mengambil konfigurasi audio default")?;

    Ok((host, device, config))
}
