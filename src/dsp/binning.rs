use rustfft::num_complex::Complex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    Logarithmic,
    Linear,
}

pub struct FrequencyBinner {
    sample_rate: f32,
    fft_size: usize,
    num_bars: usize,
    bin_ranges: Vec<(usize, usize)>,
    scale_mode: ScaleMode,
}

impl FrequencyBinner {
    pub fn new(sample_rate: f32, fft_size: usize, num_bars: usize, scale_mode: ScaleMode) -> Self {
        let mut binner = Self {
            sample_rate,
            fft_size,
            num_bars,
            bin_ranges: Vec::with_capacity(num_bars),
            scale_mode,
        };
        binner.recalculate_ranges();
        binner
    }

    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
        self.recalculate_ranges();
    }

    pub fn scale_mode(&self) -> ScaleMode {
        self.scale_mode
    }

    fn recalculate_ranges(&mut self) {
        self.bin_ranges.clear();
        let nyquist = self.sample_rate / 2.0;

        match self.scale_mode {
            ScaleMode::Logarithmic => {
                let min_freq = 20.0_f32;
                let max_freq = nyquist.min(20000.0);

                for i in 0..self.num_bars {
                    let f_start = min_freq * (max_freq / min_freq).powf(i as f32 / self.num_bars as f32);
                    let f_end = min_freq * (max_freq / min_freq).powf((i + 1) as f32 / self.num_bars as f32);

                    let k_start = ((f_start / self.sample_rate) * self.fft_size as f32).floor() as usize;
                    let k_end = (((f_end / self.sample_rate) * self.fft_size as f32).ceil() as usize)
                        .max(k_start + 1)
                        .min(self.fft_size / 2);

                    self.bin_ranges.push((k_start, k_end));
                }
            }
            ScaleMode::Linear => {
                let half_size = self.fft_size / 2;
                let chunk = (half_size / self.num_bars).max(1);

                for i in 0..self.num_bars {
                    let start = i * chunk;
                    let end = (start + chunk).min(half_size);
                    self.bin_ranges.push((start, end));
                }
            }
        }
    }

    pub fn compute_bins(&self, complex_spectrum: &[Complex<f32>], output_bars: &mut [f32], gain_db: f32) {
        let norm_factor = 1.0 / self.fft_size as f32;

        for (i, &(start, end)) in self.bin_ranges.iter().enumerate() {
            if i >= output_bars.len() {
                break;
            }

            let mut max_mag = 0.0_f32;
            for k in start..end {
                if k < complex_spectrum.len() {
                    let mag = complex_spectrum[k].norm() * norm_factor;
                    if mag > max_mag {
                        max_mag = mag;
                    }
                }
            }

            // Convert magnitude to dBFS scale: 20 * log10(magnitude) + gain
            let db = 20.0 * (max_mag + 1e-6).log10() + gain_db;
            // Map dB range [-60.0, 0.0] to normalized range [0.0, 1.0]
            let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
            output_bars[i] = normalized;
        }
    }
}
