use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};

pub struct WaveformWidget<'a> {
    samples: &'a [f32],
    theme: &'a Theme,
    title: &'a str,
}

impl<'a> WaveformWidget<'a> {
    pub fn new(samples: &'a [f32], theme: &'a Theme, title: &'a str) -> Self {
        Self { samples, theme, title }
    }
}

impl<'a> Widget for WaveformWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 || self.samples.is_empty() {
            return;
        }

        let mid_y = inner.y + inner.height / 2;
        let amplitude_scale = ((inner.height / 2) as f32).max(1.0);
        let step = (self.samples.len() as f32 / inner.width as f32).max(1.0);

        // Render zero-crossing reference line
        for x in 0..inner.width {
            if let Some(cell) = buf.cell_mut((inner.x + x, mid_y)) {
                cell.set_symbol("─").set_style(Style::default().fg(ratatui::style::Color::DarkGray));
            }
        }

        // Render waveform points
        for x in 0..inner.width {
            let sample_idx = (x as f32 * step) as usize;
            if sample_idx < self.samples.len() {
                let sample_val = self.samples[sample_idx].clamp(-1.0, 1.0);
                let y_offset = (sample_val * amplitude_scale).round() as i16;
                let target_y = (mid_y as i16 - y_offset).clamp(inner.y as i16, (inner.y + inner.height - 1) as i16) as u16;

                let abs_val = sample_val.abs();
                let color = if abs_val < 0.3 {
                    self.theme.low_color
                } else if abs_val < 0.7 {
                    self.theme.mid_color
                } else {
                    self.theme.high_color
                };

                let symbol = if sample_val > 0.1 {
                    "▲"
                } else if sample_val < -0.1 {
                    "▼"
                } else {
                    "●"
                };

                if let Some(cell) = buf.cell_mut((inner.x + x, target_y)) {
                    cell.set_symbol(symbol).set_style(Style::default().fg(color));
                }
            }
        }
    }
}
