use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

pub struct SpectrumWidget<'a> {
    bars: &'a [f32],
    peaks: &'a [f32],
    theme: &'a Theme,
    title: &'a str,
}

impl<'a> SpectrumWidget<'a> {
    pub fn new(bars: &'a [f32], peaks: &'a [f32], theme: &'a Theme, title: &'a str) -> Self {
        Self {
            bars,
            peaks,
            theme,
            title,
        }
    }
}

impl<'a> Widget for SpectrumWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.width < 4 || inner_area.height < 3 {
            return;
        }

        // Reserve bottom line for frequency axis markers
        let vis_height = inner_area.height - 1;
        let axis_y = inner_area.y + inner_area.height - 1;

        let num_bars = self.bars.len().min(inner_area.width as usize);
        if num_bars == 0 {
            return;
        }

        let bar_width = (inner_area.width as usize / num_bars).max(1);
        let max_eighths = (vis_height as f32 * 8.0) as u16;

        let sub_blocks = [" ", " ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

        for (i, (&bar_val, &peak_val)) in self.bars.iter().zip(self.peaks.iter()).take(num_bars).enumerate() {
            let x = inner_area.x + (i * bar_width) as u16;

            let total_eighths = ((bar_val.clamp(0.0, 1.0) * max_eighths as f32).round() as u16).min(max_eighths);
            let full_blocks = total_eighths / 8;
            let remainder = (total_eighths % 8) as usize;

            let peak_eighths = ((peak_val.clamp(0.0, 1.0) * max_eighths as f32).round() as u16).min(max_eighths);
            let peak_row = if peak_eighths > 0 { (peak_eighths - 1) / 8 } else { 0 };

            // Render bar columns
            for y_offset in 0..vis_height {
                let y = inner_area.y + vis_height - 1 - y_offset;
                if x >= inner_area.x + inner_area.width || y >= axis_y {
                    continue;
                }

                let height_ratio = y_offset as f32 / vis_height as f32;
                let color = if height_ratio < 0.5 {
                    self.theme.low_color
                } else if height_ratio < 0.8 {
                    self.theme.mid_color
                } else {
                    self.theme.high_color
                };

                let symbol = if y_offset < full_blocks {
                    "█"
                } else if y_offset == full_blocks && remainder > 0 {
                    sub_blocks[remainder]
                } else {
                    " "
                };

                if symbol != " " {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(symbol).set_style(Style::default().fg(color));
                    }
                }
            }

            // Render Peak Cap
            if peak_eighths > 0 && peak_row < vis_height {
                let y_peak = inner_area.y + vis_height - 1 - peak_row;
                if x < inner_area.x + inner_area.width && y_peak < axis_y {
                    if let Some(cell) = buf.cell_mut((x, y_peak)) {
                        cell.set_symbol("▔").set_style(Style::default().fg(self.theme.peak_color).add_modifier(Modifier::BOLD));
                    }
                }
            }
        }

        // Render Frequency Axis Markers at bottom line
        let axis_str = " 20Hz     100Hz     500Hz     1kHz     5kHz     10kHz    20kHz ";
        for (idx, ch) in axis_str.chars().enumerate() {
            let x = inner_area.x + idx as u16;
            if x < inner_area.x + inner_area.width {
                if let Some(cell) = buf.cell_mut((x, axis_y)) {
                    cell.set_symbol(&ch.to_string()).set_style(Style::default().fg(ratatui::style::Color::DarkGray));
                }
            }
        }
    }
}
