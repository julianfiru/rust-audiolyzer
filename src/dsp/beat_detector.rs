use std::time::Instant;

pub struct BeatDetector {
    energy_history: Vec<f32>,
    history_idx: usize,
    history_len: usize,
    last_beat_time: Instant,
    bpm_history: Vec<f32>,
    bpm_idx: usize,
    current_bpm: f32,
}

impl BeatDetector {
    pub fn new() -> Self {
        Self {
            energy_history: vec![0.0; 60],
            history_idx: 0,
            history_len: 60,
            last_beat_time: Instant::now(),
            bpm_history: vec![0.0; 4],
            bpm_idx: 0,
            current_bpm: 0.0,
        }
    }

    pub fn process(&mut self, bars: &[f32]) -> (Option<f32>, bool) {
        if bars.is_empty() {
            return (None, false);
        }

        // Calculate bass energy (first 4 bins roughly cover sub-bass and kick region)
        let num_bass_bins = std::cmp::min(4, bars.len());
        let bass_energy: f32 = bars.iter().take(num_bass_bins).sum::<f32>() / num_bass_bins as f32;
        
        let mut avg_energy = self.energy_history.iter().sum::<f32>() / self.history_len as f32;
        if avg_energy < 0.001 { avg_energy = 0.001; }

        let mut is_beat = false;

        let now = Instant::now();
        let time_since_last_beat = now.duration_since(self.last_beat_time).as_secs_f32();

        // 1.5x ratio threshold, 200ms debounce (max 300 BPM), minimum absolute energy 0.05
        if bass_energy > avg_energy * 1.5 && time_since_last_beat > 0.2 && bass_energy > 0.05 {
            is_beat = true;
            self.last_beat_time = now;
            
            let bpm = 60.0 / time_since_last_beat;
            
            // Simple sanity filter for typical music BPM (60 - 200 BPM)
            if bpm >= 60.0 && bpm <= 200.0 {
                self.bpm_history[self.bpm_idx] = bpm;
                self.bpm_idx = (self.bpm_idx + 1) % self.bpm_history.len();
                
                let valid_bpms: Vec<f32> = self.bpm_history.iter().filter(|&&b| b > 0.0).copied().collect();
                if !valid_bpms.is_empty() {
                    self.current_bpm = valid_bpms.iter().sum::<f32>() / valid_bpms.len() as f32;
                }
            }
        }

        self.energy_history[self.history_idx] = bass_energy;
        self.history_idx = (self.history_idx + 1) % self.history_len;

        let bpm_out = if self.current_bpm > 0.0 { Some(self.current_bpm) } else { None };
        (bpm_out, is_beat)
    }
}
