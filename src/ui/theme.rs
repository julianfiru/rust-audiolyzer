use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Cyberpunk,
    Matrix,
    Fire,
    StudioDark,
}

pub struct Theme {
    pub mode: ThemeMode,
    pub name: &'static str,
    pub low_color: Color,
    pub mid_color: Color,
    pub high_color: Color,
    pub peak_color: Color,
    pub accent: Color,
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Cyberpunk => Self {
                mode,
                name: "Cyberpunk Neon",
                low_color: Color::Rgb(0, 245, 255),
                mid_color: Color::Rgb(255, 0, 127),
                high_color: Color::Rgb(255, 230, 0),
                peak_color: Color::White,
                accent: Color::Cyan,
            },
            ThemeMode::Matrix => Self {
                mode,
                name: "Matrix Code",
                low_color: Color::Rgb(0, 100, 0),
                mid_color: Color::Rgb(0, 200, 0),
                high_color: Color::Rgb(50, 255, 50),
                peak_color: Color::Rgb(200, 255, 200),
                accent: Color::Green,
            },
            ThemeMode::Fire => Self {
                mode,
                name: "Fire Inferno",
                low_color: Color::Rgb(180, 0, 0),
                mid_color: Color::Rgb(255, 120, 0),
                high_color: Color::Rgb(255, 230, 0),
                peak_color: Color::White,
                accent: Color::Yellow,
            },
            ThemeMode::StudioDark => Self {
                mode,
                name: "Studio Dark",
                low_color: Color::Rgb(50, 100, 200),
                mid_color: Color::Rgb(100, 150, 250),
                high_color: Color::Rgb(180, 200, 255),
                peak_color: Color::Rgb(240, 240, 255),
                accent: Color::Blue,
            },
        }
    }

    pub fn next(&self) -> Self {
        let next_mode = match self.mode {
            ThemeMode::Cyberpunk => ThemeMode::Matrix,
            ThemeMode::Matrix => ThemeMode::Fire,
            ThemeMode::Fire => ThemeMode::StudioDark,
            ThemeMode::StudioDark => ThemeMode::Cyberpunk,
        };
        Self::new(next_mode)
    }
}
