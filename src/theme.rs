use ratatui::style::{Color, Modifier, Style};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn cycle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Dark => "DARK",
            Theme::Light => "LIGHT",
        }
    }

    // -- Background applied to the entire terminal frame --

    pub fn bg(self) -> Color {
        match self {
            Theme::Dark => Color::Black,
            Theme::Light => Color::Rgb(245, 239, 220), // warm beige
        }
    }

    // -- Top bar --

    pub fn top_bar_style(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::Rgb(245, 239, 220))
                .bg(Color::Rgb(90, 75, 55))
                .add_modifier(Modifier::BOLD),
        }
    }

    // -- Chapter tabs --

    pub fn tab_highlight_style(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::Rgb(180, 120, 20))
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn tab_border_style(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::DarkGray),
            Theme::Light => Style::default().fg(Color::Rgb(190, 180, 160)),
        }
    }

    pub fn tab_normal_style(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::Gray),
            Theme::Light => Style::default().fg(Color::Rgb(120, 110, 95)),
        }
    }

    // -- Chapter title --

    pub fn chapter_title_style(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::Rgb(50, 40, 30))
                .add_modifier(Modifier::BOLD),
        }
    }

    // -- Verse styles --

    pub fn verse_visual_cursor(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(100, 130, 180))
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn verse_visual(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::White).bg(Color::Blue),
            Theme::Light => Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(130, 155, 200)),
        }
    }

    pub fn verse_cursor_highlighted(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::Rgb(40, 35, 20))
                .bg(Color::Rgb(240, 210, 100))
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn verse_cursor(self) -> Style {
        match self {
            Theme::Dark => Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            Theme::Light => Style::default()
                .fg(Color::Rgb(30, 25, 15))
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn verse_highlighted(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::Black).bg(Color::Yellow),
            Theme::Light => Style::default()
                .fg(Color::Rgb(50, 40, 25))
                .bg(Color::Rgb(245, 220, 120)),
        }
    }

    pub fn verse_normal(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::Gray),
            Theme::Light => Style::default().fg(Color::Rgb(80, 75, 65)),
        }
    }

    // -- Status bar --

    pub fn status_bar_style(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::White).bg(Color::DarkGray),
            Theme::Light => Style::default()
                .fg(Color::Rgb(60, 55, 45))
                .bg(Color::Rgb(220, 212, 195)),
        }
    }

    // -- Help bar --

    pub fn help_bar_style(self) -> Style {
        match self {
            Theme::Dark => Style::default().fg(Color::DarkGray).bg(Color::Black),
            Theme::Light => Style::default()
                .fg(Color::Rgb(160, 150, 135))
                .bg(Color::Rgb(235, 228, 210)),
        }
    }

    // -- Persistence --

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("bible-tui").join("theme"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Theme::Dark;
        };
        match fs::read_to_string(path) {
            Ok(s) => match s.trim() {
                "light" => Theme::Light,
                _ => Theme::Dark,
            },
            Err(_) => Theme::Dark,
        }
    }

    pub fn save(self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let val = match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        };
        let _ = fs::write(path, val);
    }
}
