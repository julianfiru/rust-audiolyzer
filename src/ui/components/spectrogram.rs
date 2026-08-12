use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};
use std::collections::VecDeque;

pub struct SpectrogramWidget<'a> {
    history: &'a VecDeque<Vec<f32>>,
    theme: &'a Theme,
    title: &'a str,
}

impl<'a> SpectrogramWidget<'a> {
    pub fn new(history: &'a VecDeque<Vec<f32>>, theme: &'a Theme, title: &'a str) -> Self {
        Self {
            history,
            theme,
            title,
        }
    }

    /// Interpolates between black, low, mid, and high colors based on intensity [0.0, 1.0]
    fn get_color(&self, intensity: f32) -> Color {
        let i = intensity.clamp(0.0, 1.0);
        
        if i < 0.1 {
            // Fade from Black to Low Color
            Self::lerp_color(Color::Rgb(0, 0, 0), self.theme.low_color, i * 10.0)
        } else if i < 0.5 {
            // Fade from Low to Mid Color
            let t = (i - 0.1) / 0.4;
            Self::lerp_color(self.theme.low_color, self.theme.mid_color, t)
        } else if i < 0.9 {
            // Fade from Mid to High Color
            let t = (i - 0.5) / 0.4;
            Self::lerp_color(self.theme.mid_color, self.theme.high_color, t)
        } else {
            // Fade from High to Peak Color
            let t = (i - 0.9) / 0.1;
            Self::lerp_color(self.theme.high_color, self.theme.peak_color, t)
        }
    }

    fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
        let (r1, g1, b1) = match c1 {
            Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
            Color::White => (255.0, 255.0, 255.0),
            Color::Cyan => (0.0, 255.0, 255.0),
            Color::Green => (0.0, 255.0, 0.0),
            Color::Yellow => (255.0, 255.0, 0.0),
            Color::Blue => (0.0, 0.0, 255.0),
            _ => (0.0, 0.0, 0.0),
        };

        let (r2, g2, b2) = match c2 {
            Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
            Color::White => (255.0, 255.0, 255.0),
            Color::Cyan => (0.0, 255.0, 255.0),
            Color::Green => (0.0, 255.0, 0.0),
            Color::Yellow => (255.0, 255.0, 0.0),
            Color::Blue => (0.0, 0.0, 255.0),
            _ => (0.0, 0.0, 0.0),
        };

        let r = r1 + (r2 - r1) * t;
        let g = g1 + (g2 - g1) * t;
        let b = b1 + (b2 - b1) * t;

        Color::Rgb(r as u8, g as u8, b as u8)
    }
}

impl<'a> Widget for SpectrogramWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.width == 0 || inner_area.height == 0 || self.history.is_empty() {
            return;
        }

        let num_bins = self.history[0].len();
        let bins_per_col = (num_bins as f32 / inner_area.width as f32).max(0.1);

        // Draw Waterfall
        // Y iterates top to bottom. Row 0 in history is newest, drawn at top.
        for y in 0..inner_area.height {
            let hist_idx = y as usize;
            if hist_idx >= self.history.len() {
                break; // No more history to draw
            }

            let row_data = &self.history[hist_idx];

            for x in 0..inner_area.width {
                // Map terminal column `x` to frequency bin `bin_idx`
                let bin_idx = (x as f32 * bins_per_col) as usize;
                let val = if bin_idx < num_bins { row_data[bin_idx] } else { 0.0 };

                let color = self.get_color(val);

                // Use a full block char to draw the pixel
                if let Some(cell) = buf.cell_mut((inner_area.x + x, inner_area.y + y)) {
                    cell.set_char('█')
                        .set_style(Style::default().fg(color));
                }
            }
        }
    }
}
