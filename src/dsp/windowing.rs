use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Hann,
    Hamming,
    Rectangular,
    BlackmanHarris,
}

pub struct WindowFunction {
    weights: Vec<f32>,
    window_type: WindowType,
}

impl WindowFunction {
    pub fn new(size: usize, window_type: WindowType) -> Self {
        let weights = match window_type {
            WindowType::Hann => (0..size)
                .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f32 / (size - 1) as f32).cos()))
                .collect(),
            WindowType::Hamming => (0..size)
                .map(|n| 0.54 - 0.46 * (2.0 * PI * n as f32 / (size - 1) as f32).cos())
                .collect(),
            WindowType::Rectangular => vec![1.0; size],
            WindowType::BlackmanHarris => (0..size)
                .map(|n| {
                    let a0 = 0.35875;
                    let a1 = 0.48829;
                    let a2 = 0.14128;
                    let a3 = 0.01168;
                    let p = 2.0 * PI * n as f32 / (size - 1) as f32;
                    a0 - a1 * p.cos() + a2 * (2.0 * p).cos() - a3 * (3.0 * p).cos()
                })
                .collect(),
        };

        Self { weights, window_type }
    }

    #[inline]
    pub fn apply(&self, input: &[f32], output: &mut [f32]) {
        for (i, (&sample, &weight)) in input.iter().zip(self.weights.iter()).enumerate() {
            output[i] = sample * weight;
        }
    }

    pub fn window_type(&self) -> WindowType {
        self.window_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hann_window_symmetry() {
        let size = 1024;
        let window = WindowFunction::new(size, WindowType::Hann);
        assert!((window.weights[0] - 0.0).abs() < 1e-4);
        assert!((window.weights[size / 2] - 1.0).abs() < 1e-2);
    }
}
