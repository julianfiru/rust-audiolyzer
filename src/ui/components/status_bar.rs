use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Paragraph, Widget},
};

pub struct StatusBarWidget<'a> {
    device_name: &'a str,
    sample_rate: u32,
    gain_db: f32,
    window_type: &'a str,
    scale_mode: &'a str,
    theme: &'a Theme,
    paused: bool,
    fft_size: usize,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(
        device_name: &'a str,
        sample_rate: u32,
        gain_db: f32,
        window_type: &'a str,
        scale_mode: &'a str,
        theme: &'a Theme,
        paused: bool,
        fft_size: usize,
    ) -> Self {
        Self {
            device_name,
            sample_rate,
            gain_db,
            window_type,
            scale_mode,
            theme,
            paused,
            fft_size,
        }
    }
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pause_status = if self.paused { " [PAUSED] " } else { " [LIVE] " };

        let status_text = format!(
            " Dev: {} | Rate: {}Hz | Gain: {:+.1}dB | Window: {} | Scale: {} | FFT: {} | Theme: {} | Status: {} | 'h' for help",
            self.device_name,
            self.sample_rate,
            self.gain_db,
            self.window_type,
            self.scale_mode,
            self.fft_size,
            self.theme.name,
            pause_status
        );

        let paragraph = Paragraph::new(status_text)
            .style(Style::default().bg(self.theme.accent).fg(ratatui::style::Color::Black).add_modifier(Modifier::BOLD));

        paragraph.render(area, buf);
    }
}
