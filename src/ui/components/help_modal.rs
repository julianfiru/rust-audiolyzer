use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct HelpModalWidget<'a> {
    theme: &'a Theme,
}

impl<'a> HelpModalWidget<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl<'a> Widget for HelpModalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let modal_text = vec![
            "--- CONTROLS & KEYBINDINGS ---",
            "",
            " [1-3] Spectrum / Waveform / VU Meter",
            " [4] 2D Waterfall Spectrogram",
            " [Tab] Cycle Color Themes (Cyberpunk, Matrix, Fire, Dark)",
            " [w] Toggle Window Function (Hann -> Rect -> Hamming)",
            " [s] Toggle Frequency Scale (Logarithmic vs Linear)",
            " [ / ] Halve or Double FFT Size (Dynamic Resolution)",
            " [d] Select Audio Device (Microphone / Output)",
            " [Up/Down] Adjust Gain Sensitivity (+3dB / -3dB)",
            " [Space] Freeze / Pause Visualizer",
            " [h / ?] Close Help Modal",
            " [q / Esc] Quit Application Safely",
        ]
        .join("\n");

        let paragraph = Paragraph::new(modal_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Help & Keyboard Shortcuts ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)),
            )
            .style(Style::default().fg(self.theme.peak_color));

        Clear.render(area, buf);
        paragraph.render(area, buf);
    }
}
