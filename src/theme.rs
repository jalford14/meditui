use ratatui::style::{Color, Modifier, Style};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Midnight,
    Parchment,
    Vesper,
}

struct Palette {
    bg: Color,
    fg: Color,
    muted: Color,
    accent: Color,
    cursor_bg: Color,
    cursor_fg: Color,
    highlight_bg: Color,
    highlight_fg: Color,
    cursor_highlight_bg: Color,
    cursor_highlight_fg: Color,
    visual_bg: Color,
    visual_cursor_bg: Color,
    header_sep: Color,
    help_bg: Color,
    help_border: Color,
}

impl Theme {
    fn palette(self) -> Palette {
        match self {
            Theme::Midnight => Palette {
                bg: Color::Rgb(20, 20, 28),
                fg: Color::Rgb(188, 182, 170),
                muted: Color::Rgb(70, 68, 62),
                accent: Color::Rgb(210, 170, 90),
                cursor_bg: Color::Rgb(30, 30, 40),
                cursor_fg: Color::Rgb(230, 225, 215),
                highlight_bg: Color::Rgb(42, 40, 25),
                highlight_fg: Color::Rgb(225, 200, 120),
                cursor_highlight_bg: Color::Rgb(50, 48, 30),
                cursor_highlight_fg: Color::Rgb(240, 215, 130),
                visual_bg: Color::Rgb(35, 50, 80),
                visual_cursor_bg: Color::Rgb(45, 65, 100),
                header_sep: Color::Rgb(50, 48, 44),
                help_bg: Color::Rgb(28, 28, 38),
                help_border: Color::Rgb(55, 53, 48),
            },
            Theme::Parchment => Palette {
                bg: Color::Rgb(248, 243, 230),
                fg: Color::Rgb(55, 48, 38),
                muted: Color::Rgb(175, 165, 148),
                accent: Color::Rgb(160, 110, 30),
                cursor_bg: Color::Rgb(238, 232, 216),
                cursor_fg: Color::Rgb(30, 25, 15),
                highlight_bg: Color::Rgb(252, 242, 200),
                highlight_fg: Color::Rgb(120, 85, 10),
                cursor_highlight_bg: Color::Rgb(248, 235, 185),
                cursor_highlight_fg: Color::Rgb(100, 70, 5),
                visual_bg: Color::Rgb(180, 200, 230),
                visual_cursor_bg: Color::Rgb(150, 175, 215),
                header_sep: Color::Rgb(200, 192, 178),
                help_bg: Color::Rgb(242, 236, 222),
                help_border: Color::Rgb(190, 180, 162),
            },
            Theme::Vesper => Palette {
                bg: Color::Rgb(28, 26, 22),
                fg: Color::Rgb(198, 190, 172),
                muted: Color::Rgb(78, 74, 65),
                accent: Color::Rgb(195, 155, 75),
                cursor_bg: Color::Rgb(38, 36, 30),
                cursor_fg: Color::Rgb(235, 228, 210),
                highlight_bg: Color::Rgb(48, 44, 25),
                highlight_fg: Color::Rgb(220, 195, 110),
                cursor_highlight_bg: Color::Rgb(55, 50, 30),
                cursor_highlight_fg: Color::Rgb(235, 210, 120),
                visual_bg: Color::Rgb(40, 55, 70),
                visual_cursor_bg: Color::Rgb(50, 70, 90),
                header_sep: Color::Rgb(55, 52, 46),
                help_bg: Color::Rgb(36, 34, 28),
                help_border: Color::Rgb(60, 56, 48),
            },
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Theme::Midnight => Theme::Parchment,
            Theme::Parchment => Theme::Vesper,
            Theme::Vesper => Theme::Midnight,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Midnight => "midnight",
            Theme::Parchment => "parchment",
            Theme::Vesper => "vesper",
        }
    }

    // -- Background --

    pub fn bg(self) -> Color {
        self.palette().bg
    }

    pub fn bg_style(self) -> Style {
        Style::default().bg(self.palette().bg)
    }

    // -- Header --

    pub fn header_day_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.muted).bg(p.bg)
    }

    pub fn header_chapter_active(self) -> Style {
        let p = self.palette();
        Style::default()
            .fg(p.accent)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_chapter_inactive(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.muted).bg(p.bg)
    }

    pub fn header_separator_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.header_sep).bg(p.bg)
    }

    // -- Chapter title --

    pub fn chapter_title_style(self) -> Style {
        let p = self.palette();
        Style::default()
            .fg(p.fg)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD)
    }

    // -- Verse styles --
    // Returns (number_style, text_style) for each verse state

    pub fn verse_styles(
        self,
        is_cursor: bool,
        is_highlighted: bool,
        is_visual: bool,
    ) -> (Style, Style) {
        let p = self.palette();

        if is_visual && is_cursor {
            let bg = p.visual_cursor_bg;
            (
                Style::default()
                    .fg(Color::White)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::White)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_visual {
            let bg = p.visual_bg;
            (
                Style::default().fg(Color::White).bg(bg),
                Style::default().fg(Color::White).bg(bg),
            )
        } else if is_cursor && is_highlighted {
            (
                Style::default()
                    .fg(p.accent)
                    .bg(p.cursor_highlight_bg)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(p.cursor_highlight_fg)
                    .bg(p.cursor_highlight_bg)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_cursor {
            (
                Style::default()
                    .fg(p.accent)
                    .bg(p.cursor_bg)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(p.cursor_fg).bg(p.cursor_bg),
            )
        } else if is_highlighted {
            (
                Style::default().fg(p.accent).bg(p.highlight_bg),
                Style::default().fg(p.highlight_fg).bg(p.highlight_bg),
            )
        } else {
            (
                Style::default().fg(p.muted).bg(p.bg),
                Style::default().fg(p.fg).bg(p.bg),
            )
        }
    }

    // -- Status bar --

    pub fn status_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.muted).bg(p.bg)
    }

    pub fn status_accent_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.accent).bg(p.bg)
    }

    // -- Help overlay --

    pub fn help_border_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.help_border).bg(p.help_bg)
    }

    pub fn help_title_style(self) -> Style {
        let p = self.palette();
        Style::default()
            .fg(p.accent)
            .bg(p.help_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn help_key_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.accent).bg(p.help_bg)
    }

    pub fn help_desc_style(self) -> Style {
        let p = self.palette();
        Style::default().fg(p.fg).bg(p.help_bg)
    }

    pub fn help_bg_style(self) -> Style {
        Style::default().bg(self.palette().help_bg)
    }

    // -- Persistence --

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("bible-tui").join("theme"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Theme::Midnight;
        };
        match fs::read_to_string(path) {
            Ok(s) => match s.trim() {
                "midnight" => Theme::Midnight,
                "parchment" => Theme::Parchment,
                "vesper" => Theme::Vesper,
                "dark" => Theme::Midnight,
                "light" => Theme::Parchment,
                _ => Theme::Midnight,
            },
            Err(_) => Theme::Midnight,
        }
    }

    pub fn save(self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, self.label());
    }
}
