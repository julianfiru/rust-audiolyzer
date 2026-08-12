use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};

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
