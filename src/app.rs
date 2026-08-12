use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::collections::VecDeque;
use crate::{
    audio::{devices, AudioStreamManager},
    dsp::DspEngine,
    ui::theme::{Theme, ThemeMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Spectrum,
    Waveform,
    VuMeter,
    Spectrogram,
}

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub show_help: bool,
    pub show_device_selector: bool,
    pub available_devices: Vec<String>,
    pub device_list_state: ListState,
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub gain_db: f32,
    pub audio_stream: AudioStreamManager,
    pub dsp_engine: DspEngine,
    pub audio_time_domain_buffer: Vec<f32>,
    pub bar_values: Vec<f32>,
    pub peak_values: Vec<f32>,
    pub spectrogram_history: VecDeque<Vec<f32>>,
    pub fft_size: usize,
    pub current_bpm: Option<f32>,
    pub beat_flash_timer: u8,
}

impl App {
    pub fn new() -> Result<Self> {
        let buffer_capacity = 16384;
        let fft_size = 2048;
        let num_bars = 48;

        let audio_stream = AudioStreamManager::new(buffer_capacity)?;
        let sample_rate = audio_stream.sample_rate() as f32;
        let dsp_engine = DspEngine::new(sample_rate, fft_size, num_bars);

        let mut spectrogram_history = VecDeque::with_capacity(120);
        for _ in 0..120 {
            spectrogram_history.push_back(vec![0.0; num_bars]);
        }

        let available_devices = devices::get_available_devices().unwrap_or_default();
        let mut device_list_state = ListState::default();
        if !available_devices.is_empty() {
            device_list_state.select(Some(0));
        }

        Ok(Self {
            running: true,
            paused: false,
            show_help: false,
            show_device_selector: false,
            available_devices,
            device_list_state,
            view_mode: ViewMode::Spectrum,
            theme: Theme::new(ThemeMode::Cyberpunk),
            gain_db: 0.0,
            audio_stream,
            dsp_engine,
            audio_time_domain_buffer: vec![0.0; fft_size],
            bar_values: vec![0.0; num_bars],
            peak_values: vec![0.0; num_bars],
            spectrogram_history,
            fft_size,
            current_bpm: None,
            beat_flash_timer: 0,
        })
    }

    pub fn update(&mut self) {
        if self.paused || self.show_device_selector || self.show_help {
            return;
        }

        if self.beat_flash_timer > 0 {
            self.beat_flash_timer -= 1;
        }

        // Pull audio samples from lock-free SPSC Consumer
        if self.audio_stream.available_samples() >= self.fft_size {
            self.audio_stream.pop_samples(&mut self.audio_time_domain_buffer);

            // Execute DSP pipeline (Hann windowing -> FFT -> Log binning -> Ballistics -> Beat)
            let (bars, peaks, bpm, is_beat) = self
                .dsp_engine
                .process_samples(&self.audio_time_domain_buffer, self.gain_db);

            self.bar_values.copy_from_slice(bars);
            self.peak_values.copy_from_slice(peaks);
            
            if is_beat {
                self.beat_flash_timer = 5; // Flash for 5 frames
            }
            if let Some(new_bpm) = bpm {
                self.current_bpm = Some(new_bpm);
            }

            // Update Spectrogram History
            self.spectrogram_history.push_front(bars.to_vec());
            if self.spectrogram_history.len() > 120 {
                self.spectrogram_history.pop_back();
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyEventKind, KeyModifiers};

        // Only trigger on key press, ignore auto-repeat and release events to prevent flickering and lagging
        if key.kind != KeyEventKind::Press {
            return;
        }

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        if self.show_device_selector {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
                    self.show_device_selector = false;
                }
                KeyCode::Up => {
                    if let Some(selected) = self.device_list_state.selected() {
                        if selected > 0 {
                            self.device_list_state.select(Some(selected - 1));
                        } else {
                            self.device_list_state.select(Some(self.available_devices.len() - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(selected) = self.device_list_state.selected() {
                        if selected < self.available_devices.len() - 1 {
                            self.device_list_state.select(Some(selected + 1));
                        } else {
                            self.device_list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.device_list_state.selected() {
                        if let Some(dev_name) = self.available_devices.get(selected) {
                            if let Ok(new_stream) = AudioStreamManager::new_with_device(dev_name, 16384) {
                                self.audio_stream = new_stream;
                                // Reset DSP with new sample rate
                                self.dsp_engine = DspEngine::new(self.audio_stream.sample_rate() as f32, self.fft_size, 48);
                                self.spectrogram_history.clear();
                                for _ in 0..120 {
                                    self.spectrogram_history.push_back(vec![0.0; 48]);
                                }
                            }
                        }
                    }
                    self.show_device_selector = false;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.available_devices = devices::get_available_devices().unwrap_or_default();
                self.show_device_selector = true;
            }
            KeyCode::Char(' ') => self.paused = !self.paused,
            KeyCode::Char('h') | KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('1') => self.view_mode = ViewMode::Spectrum,
            KeyCode::Char('2') => self.view_mode = ViewMode::Waveform,
            KeyCode::Char('3') => self.view_mode = ViewMode::VuMeter,
            KeyCode::Char('4') => self.view_mode = ViewMode::Spectrogram,
            KeyCode::Tab => self.theme = self.theme.next(),
            KeyCode::Char('w') => {
                self.dsp_engine.toggle_windowing();
            }
            KeyCode::Char('s') => {
                self.dsp_engine.toggle_scale_mode();
            }
            KeyCode::Up => self.gain_db = (self.gain_db + 3.0).min(24.0),
            KeyCode::Down => self.gain_db = (self.gain_db - 3.0).max(-24.0),
            KeyCode::Char('[') => {
                if self.fft_size > 512 {
                    self.fft_size /= 2;
                    self.audio_time_domain_buffer = vec![0.0; self.fft_size];
                    self.dsp_engine = DspEngine::new(self.audio_stream.sample_rate() as f32, self.fft_size, 48);
                    self.spectrogram_history.clear();
                    for _ in 0..120 {
                        self.spectrogram_history.push_back(vec![0.0; 48]);
                    }
                }
            }
            KeyCode::Char(']') => {
                if self.fft_size < 8192 {
                    self.fft_size *= 2;
                    self.audio_time_domain_buffer = vec![0.0; self.fft_size];
                    self.dsp_engine = DspEngine::new(self.audio_stream.sample_rate() as f32, self.fft_size, 48);
                    self.spectrogram_history.clear();
                    for _ in 0..120 {
                        self.spectrogram_history.push_back(vec![0.0; 48]);
                    }
                }
            }
            _ => {}
        }
    }
}
