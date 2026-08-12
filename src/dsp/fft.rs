use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

pub struct FftProcessor {
    fft: Arc<dyn Fft<f32>>,
    complex_buffer: Vec<Complex<f32>>,
    scratch_buffer: Vec<Complex<f32>>,
    fft_size: usize,
}

impl FftProcessor {
    pub fn new(fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let scratch_len = fft.get_inplace_scratch_len();

        Self {
            fft,
            complex_buffer: vec![Complex { re: 0.0, im: 0.0 }; fft_size],
            scratch_buffer: vec![Complex { re: 0.0, im: 0.0 }; scratch_len],
            fft_size,
        }
    }

    /// Zero-allocation FFT execution on input slice
    pub fn process(&mut self, windowed_samples: &[f32]) -> &[Complex<f32>] {
        for (i, &sample) in windowed_samples.iter().take(self.fft_size).enumerate() {
            self.complex_buffer[i] = Complex { re: sample, im: 0.0 };
        }

        self.fft.process_with_scratch(&mut self.complex_buffer, &mut self.scratch_buffer);
        &self.complex_buffer[..self.fft_size / 2]
    }

    #[allow(dead_code)]
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
}
