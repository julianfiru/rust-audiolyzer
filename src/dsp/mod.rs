pub mod ballistics;
pub mod binning;
pub mod fft;
pub mod windowing;

use ballistics::BallisticsEngine;
use binning::{FrequencyBinner, ScaleMode};
use fft::FftProcessor;
use windowing::{WindowFunction, WindowType};

pub struct DspEngine {
    window_fn: WindowFunction,
    fft_processor: FftProcessor,
    binner: FrequencyBinner,
    ballistics: BallisticsEngine,
    windowed_buffer: Vec<f32>,
    raw_bar_targets: Vec<f32>,
    fft_size: usize,
    _num_bars: usize,
}

impl DspEngine {
    pub fn new(sample_rate: f32, fft_size: usize, num_bars: usize) -> Self {
        Self {
            window_fn: WindowFunction::new(fft_size, WindowType::Hann),
            fft_processor: FftProcessor::new(fft_size),
            binner: FrequencyBinner::new(sample_rate, fft_size, num_bars, ScaleMode::Logarithmic),
            ballistics: BallisticsEngine::new(num_bars, 0.82, 0.03),
            windowed_buffer: vec![0.0; fft_size],
            raw_bar_targets: vec![0.0; num_bars],
            fft_size,
            _num_bars: num_bars,
        }
    }

    /// Process raw audio PCM samples through the full DSP pipeline
    pub fn process_samples(&mut self, samples: &[f32], gain_db: f32) -> (&[f32], &[f32]) {
        // 1. Apply Window Function (Hann)
        self.window_fn.apply(samples, &mut self.windowed_buffer);

        // 2. Perform Forward FFT
        let complex_spectrum = self.fft_processor.process(&self.windowed_buffer);

        // 3. Bin Spectrum into Frequency Bands & Convert to dBFS Scale
        self.binner.compute_bins(complex_spectrum, &mut self.raw_bar_targets, gain_db);

        // 4. Apply Ballistics Decay & Peak Detection
        self.ballistics.update(&self.raw_bar_targets)
    }

    pub fn toggle_windowing(&mut self) -> WindowType {
        let next_type = match self.window_fn.window_type() {
            WindowType::Hann => WindowType::Rectangular,
            WindowType::Rectangular => WindowType::Hamming,
            WindowType::Hamming => WindowType::Hann,
        };
        self.window_fn = WindowFunction::new(self.fft_size, next_type);
        next_type
    }

    pub fn toggle_scale_mode(&mut self) -> ScaleMode {
        let next_scale = match self.binner.scale_mode() {
            ScaleMode::Logarithmic => ScaleMode::Linear,
            ScaleMode::Linear => ScaleMode::Logarithmic,
        };
        self.binner.set_scale_mode(next_scale);
        next_scale
    }

    pub fn window_type(&self) -> WindowType {
        self.window_fn.window_type()
    }

    pub fn scale_mode(&self) -> ScaleMode {
        self.binner.scale_mode()
    }

    #[allow(dead_code)]
    pub fn num_bars(&self) -> usize {
        self._num_bars
    }
}
