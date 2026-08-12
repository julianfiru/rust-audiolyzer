use crate::ui::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};

pub struct DeviceSelectorWidget<'a> {
    devices: &'a [String],
    theme: &'a Theme,
}

impl<'a> DeviceSelectorWidget<'a> {
    pub fn new(devices: &'a [String], theme: &'a Theme) -> Self {
        Self { devices, theme }
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ].as_ref())
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ].as_ref())
            .split(popup_layout[1])[1]
    }
}

use ratatui::widgets::StatefulWidget;

impl<'a> StatefulWidget for DeviceSelectorWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = Self::centered_rect(60, 60, area);

        // Clear background
        Clear.render(area, buf);

        let block = Block::default()
            .title(" Select Audio Device (Up/Down to navigate, Enter to select, D/Esc to cancel) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));

        if self.devices.is_empty() {
            let p = Paragraph::new("No audio devices found!").block(block);
            p.render(area, buf);
            return;
        }

        let items: Vec<ListItem> = self
            .devices
            .iter()
            .map(|d| ListItem::new(d.clone()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(self.theme.accent)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, state);
    }
}
