pub mod components;
pub mod theme;

use components::{
    help_modal::HelpModalWidget, spectrogram::SpectrogramWidget, spectrum::SpectrumWidget, status_bar::StatusBarWidget,
    vu_meter::VuMeterWidget, waveform::WaveformWidget,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &mut App) {
    // 1. Vertical Layout: Top Header (3), Main Body (Min 10), Bottom Dock (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ].as_ref())
        .split(frame.area());

    let header_area = chunks[0];
    let body_area = chunks[1];
    let footer_area = chunks[2];

    let window_str = format!("{:?}", app.dsp_engine.window_type());
    let scale_str = format!("{:?}", app.dsp_engine.scale_mode());

    // --- A. RENDER HEADER WITH MODE TABS & STATUS ---
    let (tab_1, tab_2, tab_3, tab_4) = match app.view_mode {
        ViewMode::Spectrum =>    (" > [1] SPEC ", "   [2] WAVE ", "   [3] VU ", "   [4] WATERFALL "),
        ViewMode::Waveform =>    ("   [1] SPEC ", " > [2] WAVE ", "   [3] VU ", "   [4] WATERFALL "),
        ViewMode::VuMeter =>     ("   [1] SPEC ", "   [2] WAVE ", " > [3] VU ", "   [4] WATERFALL "),
        ViewMode::Spectrogram => ("   [1] SPEC ", "   [2] WAVE ", "   [3] VU ", " > [4] WATERFALL "),
    };

    let status_badge = if app.paused { " [PAUSED] " } else { " [LIVE] " };

    let header_text = format!(
        "AUDIOLYZER Pro  | {}{}{}{} | Theme: {} | Status: {}",
        tab_1, tab_2, tab_3, tab_4, app.theme.name, status_badge
    );

    let header_widget = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().fg(app.theme.peak_color));

    frame.render_widget(header_widget, header_area);

    // --- B. RENDER MAIN BODY (Visualizer + Side Dashboard) ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(78), Constraint::Percentage(22)].as_ref())
        .split(body_area);

    let visualizer_area = body_chunks[0];
    let sidebar_area = body_chunks[1];

    // 1. Main View Mode
    match app.view_mode {
        ViewMode::Spectrum => {
            let widget = SpectrumWidget::new(
                &app.bar_values,
                &app.peak_values,
                &app.theme,
                "Real-Time Audio Spectrum Analyzer (Sub-Cell 8x Precision)",
            );
            frame.render_widget(widget, visualizer_area);
        }
        ViewMode::Waveform => {
            let widget = WaveformWidget::new(
                &app.audio_time_domain_buffer,
                &app.theme,
                "Time-Domain Oscilloscope (PCM Signal)",
            );
            frame.render_widget(widget, visualizer_area);
        }
        ViewMode::VuMeter => {
            let max_val = app.bar_values.iter().cloned().fold(0.0_f32, f32::max);
            let peak_val = app.peak_values.iter().cloned().fold(0.0_f32, f32::max);
            let widget = VuMeterWidget::new(
                max_val,
                peak_val,
                &app.theme,
                "Stereo Master Peak & RMS Level Meter",
            );
            frame.render_widget(widget, visualizer_area);
        }
        ViewMode::Spectrogram => {
            let widget = SpectrogramWidget::new(
                &app.spectrogram_history,
                &app.theme,
                "2D Waterfall Spectrogram",
            );
            frame.render_widget(widget, visualizer_area);
        }
    }

    // 2. Side Audio Metrics Dashboard Panel
    let max_bar = app.bar_values.iter().cloned().fold(0.0_f32, f32::max);
    let peak_db = if max_bar > 1e-5 { 20.0 * max_bar.log10() + app.gain_db } else { -60.0 };

    let mut dev_name = app.audio_stream.device_name().to_string();
    if dev_name.chars().count() > 20 {
        dev_name = dev_name.chars().take(20).collect();
        dev_name.push_str("...");
    }

    let bpm_str = match app.current_bpm {
        Some(bpm) => format!("{:>3.0} BPM", bpm),
        None => "Detecting...".to_string(),
    };

    let beat_hit_str = if app.beat_flash_timer > 0 {
        "  [BASS HIT!] "
    } else {
        "              "
    };

    let sidebar_text = vec![
        format!("Device:"),
        format!(" {}", dev_name),
        format!(""),
        format!("Gain   : {:+.1} dB", app.gain_db),
        format!("Window : {}", window_str),
        format!("Scale  : {}", scale_str),
        format!("Peak   : {:>5.1} dBFS", peak_db),
        format!("Rate   : {} Hz", app.audio_stream.sample_rate()),
        format!(""),
        format!("Tempo  : {}", bpm_str),
        format!("{}", beat_hit_str),
        format!(""),
        format!("[H] Help"),
    ].join("\n");

    let mut border_style = Style::default().fg(app.theme.accent);
    // Flash the border color when a beat hits
    if app.beat_flash_timer > 0 {
        border_style = Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD);
    }

    let sidebar_widget = Paragraph::new(sidebar_text)
        .block(
            Block::default()
                .title(" Metrics ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(app.theme.peak_color));

    frame.render_widget(sidebar_widget, sidebar_area);

    // --- C. RENDER BOTTOM DOCK SHORTCUT BAR ---
    let status_bar = StatusBarWidget::new(
        app.audio_stream.device_name(),
        app.audio_stream.sample_rate(),
        app.gain_db,
        &window_str,
        &scale_str,
        &app.theme,
        app.paused,
        app.fft_size,
    );
    frame.render_widget(status_bar, footer_area);

    // --- D. RENDER HELP MODAL OVERLAY ---
    if app.show_help {
        let modal_area = centered_rect(60, 60, frame.area());
        let modal = HelpModalWidget::new(&app.theme);
        frame.render_widget(modal, modal_area);
    }

    // --- E. RENDER DEVICE SELECTOR MODAL ---
    if app.show_device_selector {
        use components::device_selector::DeviceSelectorWidget;
        let widget = DeviceSelectorWidget::new(&app.available_devices, &app.theme);
        frame.render_stateful_widget(widget, frame.area(), &mut app.device_list_state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
