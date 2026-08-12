use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use ringbuf::{Consumer, HeapRb};

pub struct AudioStreamManager {
    _stream: cpal::Stream,
    consumer: Consumer<f32, std::sync::Arc<HeapRb<f32>>>,
    sample_rate: u32,
    _channels: u16,
    device_name: String,
}

impl AudioStreamManager {
    pub fn new(buffer_capacity: usize) -> Result<Self> {
        let (_host, device, supported_config) = super::devices::get_default_loopback_device()?;
        let device_name = device.name().unwrap_or_else(|_| "Default Output (System Audio)".to_string());
        let sample_rate = supported_config.sample_rate().0;
        let _channels = supported_config.channels();
        let sample_format = supported_config.sample_format();
        let config: cpal::StreamConfig = supported_config.into();

        let rb = HeapRb::<f32>::new(buffer_capacity);
        let (mut producer, consumer) = rb.split();

        let err_fn = |err| eprintln!("Audio stream callback error: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let _ = producer.push_slice(data);
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let _ = producer.push_slice(&f32_data);
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0)).collect();
                    let _ = producer.push_slice(&f32_data);
                },
                err_fn,
                None,
            )?,
            sample_format => return Err(anyhow::anyhow!("Unsupported sample format '{sample_format}'")),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            consumer,
            sample_rate,
            _channels,
            device_name,
        })
    }

    pub fn pop_samples(&mut self, output_buffer: &mut [f32]) -> usize {
        self.consumer.pop_slice(output_buffer)
    }

    pub fn available_samples(&self) -> usize {
        self.consumer.len()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    #[allow(dead_code)]
    pub fn channels(&self) -> u16 {
        self._channels
    }
}
