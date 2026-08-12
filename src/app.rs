use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use crate::{
    audio::AudioStreamManager,
    dsp::DspEngine,
    ui::theme::{Theme, ThemeMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Spectrum,
    Waveform,
    VuMeter,
}

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub show_help: bool,
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub gain_db: f32,
    pub audio_stream: AudioStreamManager,
    pub dsp_engine: DspEngine,
    pub audio_time_domain_buffer: Vec<f32>,
    pub bar_values: Vec<f32>,
    pub peak_values: Vec<f32>,
    pub fft_size: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let buffer_capacity = 16384;
        let fft_size = 2048;
        let num_bars = 48;

        let audio_stream = AudioStreamManager::new(buffer_capacity)?;
        let sample_rate = audio_stream.sample_rate() as f32;
        let dsp_engine = DspEngine::new(sample_rate, fft_size, num_bars);

        Ok(Self {
            running: true,
            paused: false,
            show_help: false,
            view_mode: ViewMode::Spectrum,
            theme: Theme::new(ThemeMode::Cyberpunk),
            gain_db: 0.0,
            audio_stream,
            dsp_engine,
            audio_time_domain_buffer: vec![0.0; fft_size],
            bar_values: vec![0.0; num_bars],
            peak_values: vec![0.0; num_bars],
            fft_size,
        })
    }

    pub fn update(&mut self) {
        if self.paused {
            return;
        }

        // Pull audio samples from lock-free SPSC Consumer
        if self.audio_stream.available_samples() >= self.fft_size {
            self.audio_stream.pop_samples(&mut self.audio_time_domain_buffer);

            // Execute DSP pipeline (Hann windowing -> FFT -> Log binning -> Ballistics)
            let (bars, peaks) = self
                .dsp_engine
                .process_samples(&self.audio_time_domain_buffer, self.gain_db);

            self.bar_values.copy_from_slice(bars);
            self.peak_values.copy_from_slice(peaks);
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::KeyModifiers;

        // Ensure Ctrl+C explicitly shuts down the app gracefully to avoid OS Error 32 zombie processes
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char(' ') => self.paused = !self.paused,
            KeyCode::Char('h') | KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('1') => self.view_mode = ViewMode::Spectrum,
            KeyCode::Char('2') => self.view_mode = ViewMode::Waveform,
            KeyCode::Char('3') => self.view_mode = ViewMode::VuMeter,
            KeyCode::Tab => self.theme = self.theme.next(),
            KeyCode::Char('w') => {
                self.dsp_engine.toggle_windowing();
            }
            KeyCode::Char('s') => {
                self.dsp_engine.toggle_scale_mode();
            }
            KeyCode::Up => self.gain_db = (self.gain_db + 3.0).min(24.0),
            KeyCode::Down => self.gain_db = (self.gain_db - 3.0).max(-24.0),
            _ => {}
        }
    }
}
