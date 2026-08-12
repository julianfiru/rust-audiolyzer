pub struct BallisticsEngine {
    current_values: Vec<f32>,
    peak_values: Vec<f32>,
    decay_rate: f32,
    peak_fall_speed: f32,
}

impl BallisticsEngine {
    pub fn new(num_bars: usize, decay_rate: f32, peak_fall_speed: f32) -> Self {
        Self {
            current_values: vec![0.0; num_bars],
            peak_values: vec![0.0; num_bars],
            decay_rate,
            peak_fall_speed,
        }
    }

    pub fn update(&mut self, targets: &[f32]) -> (&[f32], &[f32]) {
        let len = targets.len().min(self.current_values.len());

        for i in 0..len {
            let target = targets[i];

            // Exponential decay smoothing for main bars
            if target >= self.current_values[i] {
                self.current_values[i] = target;
            } else {
                self.current_values[i] = (self.current_values[i] * self.decay_rate).max(target);
            }

            // Peak cap logic with gravity falloff
            if self.current_values[i] >= self.peak_values[i] {
                self.peak_values[i] = self.current_values[i];
            } else {
                self.peak_values[i] = (self.peak_values[i] - self.peak_fall_speed).max(0.0);
            }
        }

        (&self.current_values[..len], &self.peak_values[..len])
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.current_values.fill(0.0);
        self.peak_values.fill(0.0);
    }
}
