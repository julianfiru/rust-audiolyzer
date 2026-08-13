use rustfft::num_complex::Complex;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct AudioAnalytics {
    pub peak_hz: f32,
    pub note_name: String,
    pub cutoff_hz: Option<f32>,
    pub is_lossy: bool,
    pub debug_info: String,
    cutoff_history: VecDeque<f32>,
}

impl Default for AudioAnalytics {
    fn default() -> Self {
        Self {
            peak_hz: 0.0,
            note_name: "-".to_string(),
            cutoff_hz: None,
            is_lossy: false,
            debug_info: String::new(),
            cutoff_history: VecDeque::with_capacity(180),
        }
    }
}

impl AudioAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dump the full magnitude spectrum to a CSV file for debugging.
    /// Call this once to inspect the raw FFT output shape.
    pub fn dump_spectrum_csv(complex_spectrum: &[Complex<f32>], sample_rate: f32, fft_size: usize) {
        use std::io::Write;
        let hz_per_bin = sample_rate / fft_size as f32;
        let path = "spectrum_debug.csv";
        if let Ok(mut f) = std::fs::File::create(path) {
            let _ = writeln!(f, "bin,frequency_hz,magnitude,magnitude_db");
            for (i, c) in complex_spectrum.iter().enumerate() {
                let mag = c.norm();
                let db = if mag > 1e-10 { 20.0 * mag.log10() } else { -200.0 };
                let _ = writeln!(f, "{},{:.2},{:.6},{:.2}", i, i as f32 * hz_per_bin, mag, db);
            }
        }
    }

    pub fn process(&mut self, complex_spectrum: &[Complex<f32>], sample_rate: f32, fft_size: usize) {
        if complex_spectrum.is_empty() {
            return;
        }

        let hz_per_bin = sample_rate / fft_size as f32;
        
        let mut max_mag = 0.0_f32;
        let mut max_idx = 0;

        // Skip DC offset (bin 0) to avoid false peaks at 0Hz
        for i in 1..complex_spectrum.len() {
            let mag = complex_spectrum[i].norm();
            if mag > max_mag {
                max_mag = mag;
                max_idx = i;
            }
        }

        // --- 1. Peak Frequency & Musical Note ---
        self.peak_hz = max_idx as f32 * hz_per_bin;
        
        if self.peak_hz > 16.0 && max_mag > 0.5 { 
            let midi_note = (12.0 * (self.peak_hz / 440.0).log2() + 69.0).round() as i32;
            let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
            
            if midi_note >= 0 && midi_note <= 127 {
                let note_idx = (midi_note % 12) as usize;
                let octave = (midi_note / 12) - 1;
                self.note_name = format!("{}{}", note_names[note_idx], octave);
            } else {
                self.note_name = "-".to_string();
            }
        } else {
            self.peak_hz = 0.0;
            self.note_name = "-".to_string();
        }

        // --- 2. Lossy Cutoff Estimator (Valley Detection) ---
        // Based on empirical analysis of the actual FFT spectrum from WASAPI loopback,
        // we discovered that Windows Audio Engine's resampler (44.1kHz -> 48kHz) creates
        // LOUD artifacts above the MP3 brickwall cutoff. The spectrum shape for a 128kbps MP3 is:
        //   0-10kHz:  Music content (loud)
        //   10-16kHz: Dead silent valley (MP3 brickwall killed everything here)
        //   16-24kHz: WASAPI resampler artifacts (can be LOUDER than music!)
        //
        // So we look for the "valley" - the quietest band. If there's a valley with
        // energy >40dB below the music content, that's the MP3 cutoff zone.
        
        if max_mag > 10.0 {
            let max_idx_limit = std::cmp::min(
                (sample_rate / 2.0 / hz_per_bin) as usize, 
                complex_spectrum.len()
            );
            
            // Step 1: Compute average energy in sliding bands of ~500Hz width
            let band_width = ((500.0 / hz_per_bin) as usize).max(2);
            let search_start = ((8000.0 / hz_per_bin) as usize).min(max_idx_limit);
            let search_end = ((22000.0 / hz_per_bin) as usize).min(max_idx_limit);
            
            // Step 2: Find the quietest band between 8kHz and 22kHz
            let mut min_band_energy = f32::MAX;
            let mut min_band_center = search_start;
            
            if search_end > search_start + band_width {
                for i in search_start..(search_end - band_width) {
                    let mut band_energy = 0.0_f32;
                    for j in 0..band_width {
                        let mag = complex_spectrum[i + j].norm();
                        band_energy += mag * mag; // Power
                    }
                    band_energy /= band_width as f32;
                    
                    if band_energy < min_band_energy {
                        min_band_energy = band_energy;
                        min_band_center = i + band_width / 2;
                    }
                }
            }
            
            // Step 3: Compare valley energy to the music content energy (1-8kHz)
            let music_start = ((1000.0 / hz_per_bin) as usize).min(max_idx_limit);
            let music_end = ((8000.0 / hz_per_bin) as usize).min(max_idx_limit);
            let mut music_energy = 0.0_f32;
            let mut music_count = 0;
            for i in music_start..music_end {
                let mag = complex_spectrum[i].norm();
                music_energy += mag * mag;
                music_count += 1;
            }
            let avg_music_power = if music_count > 0 { music_energy / music_count as f32 } else { 1.0 };
            
            // Step 4: Two criteria MUST BOTH be true to declare lossy:
            //
            // A) The valley must be >50dB quieter than music content (100000x power ratio)
            //    This eliminates natural spectral rolloff which is typically only 30-40dB.
            //
            // B) The valley's absolute average magnitude must be < 0.001 (~-60dB)
            //    This is the critical insight from comparing real data:
            //      - MP3 dead zone: magnitude ~0.000002 to 0.000005 (-110dB) → BELOW 0.001
            //      - AIFF quiet band: magnitude ~0.02 to 0.1 (-34 to -20dB) → ABOVE 0.001
            //    No real music content (even very quiet) drops to 0.001 across an entire 500Hz band.
            //    Only a lossy codec's brickwall filter creates that absolute silence.
            
            let power_ratio = if min_band_energy > 0.0 { avg_music_power / min_band_energy } else { 0.0 };
            let valley_drop_db = if power_ratio > 0.0 { 10.0 * power_ratio.log10() } else { 0.0 };
            let valley_avg_mag = min_band_energy.sqrt(); // Convert avg power back to magnitude
            
            let is_dead_zone = valley_drop_db > 50.0 && valley_avg_mag < 0.001;
            
            let inst_cutoff = if is_dead_zone {
                // The valley center IS the cutoff zone
                min_band_center as f32 * hz_per_bin
            } else {
                sample_rate / 2.0 // No dead zone = lossless
            };

            self.debug_info = format!("v:{:.0}dB m:{:.6} @{:.0}Hz", valley_drop_db, valley_avg_mag, min_band_center as f32 * hz_per_bin);

            // Store in 3-second Rolling Window Memory (180 frames at 60 FPS)
            self.cutoff_history.push_back(inst_cutoff);
            if self.cutoff_history.len() > 180 {
                self.cutoff_history.pop_front();
            }

            // Use MEDIAN of the last 3 seconds for stability
            let mut sorted: Vec<f32> = self.cutoff_history.iter().cloned().collect();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median_cutoff = sorted[sorted.len() / 2];

            self.cutoff_hz = Some(median_cutoff);
            self.is_lossy = median_cutoff < 20500.0;
        } else {
            self.cutoff_hz = None;
            self.is_lossy = false;
            self.debug_info = "quiet".to_string();
        }
    }
}
