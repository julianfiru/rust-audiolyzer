use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};

pub struct VuMeterWidget<'a> {
    rms_level: f32,
    peak_level: f32,
    theme: &'a Theme,
    title: &'a str,
}

impl<'a> VuMeterWidget<'a> {
    pub fn new(rms_level: f32, peak_level: f32, theme: &'a Theme, title: &'a str) -> Self {
        Self {
            rms_level,
            peak_level,
            theme,
            title,
        }
    }
}

impl<'a> Widget for VuMeterWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 4 {
            return;
        }

        let rms_db = if self.rms_level > 1e-5 { 20.0 * self.rms_level.log10() } else { -60.0 };
        let peak_db = if self.peak_level > 1e-5 { 20.0 * self.peak_level.log10() } else { -60.0 };

        let total_blocks = (inner.width - 8) as f32; // Leave room for channel label
        let filled_blocks = ((self.rms_level.clamp(0.0, 1.0) * total_blocks).round() as u16).min(total_blocks as u16);
        let peak_block = ((self.peak_level.clamp(0.0, 1.0) * total_blocks).round() as u16).min(total_blocks as u16);

        // Row 1: Left Channel
        let y_l = inner.y + 1;
        // Row 2: Right Channel
        let y_r = inner.y + 2;

        let render_bar = |y: u16, ch_label: &str, buf: &mut Buffer| {
            // Label
            for (i, c) in ch_label.chars().enumerate() {
                if let Some(cell) = buf.cell_mut((inner.x + i as u16, y)) {
                    cell.set_symbol(&c.to_string()).set_style(Style::default().fg(self.theme.accent));
                }
            }

            let start_x = inner.x + 6;
            for x_offset in 0..total_blocks as u16 {
                let x = start_x + x_offset;
                let ratio = x_offset as f32 / total_blocks;

                let color = if ratio < 0.6 {
                    self.theme.low_color
                } else if ratio < 0.85 {
                    self.theme.mid_color
                } else {
                    self.theme.high_color
                };

                if x_offset < filled_blocks {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol("█").set_style(Style::default().fg(color));
                    }
                } else if x_offset == peak_block {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol("│").set_style(Style::default().fg(self.theme.peak_color));
                    }
                } else {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol("░").set_style(Style::default().fg(ratatui::style::Color::DarkGray));
                    }
                }
            }
        };

        render_bar(y_l, " [L] ", buf);
        render_bar(y_r, " [R] ", buf);

        // Readout text at bottom
        let readout_y = inner.y + inner.height - 1;
        let readout_text = format!(
            " RMS Level: {:>5.1} dB | Peak Level: {:>5.1} dB | Clipping: {} ",
            rms_db,
            peak_db,
            if self.peak_level >= 0.98 { "YES (RED)" } else { "NO (OK)" }
        );

        for (idx, ch) in readout_text.chars().enumerate() {
            let x = inner.x + idx as u16;
            if x < inner.x + inner.width {
                if let Some(cell) = buf.cell_mut((x, readout_y)) {
                    cell.set_symbol(&ch.to_string()).set_style(Style::default().fg(self.theme.peak_color));
                }
            }
        }
    }
}
