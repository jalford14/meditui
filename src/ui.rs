use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::animation::lerp_color;
use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.theme;

    // Fill entire background
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(theme.bg())),
        area,
    );

    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Min(1),   // verses
        Constraint::Length(1), // status
    ])
    .split(area);

    draw_header(f, app, chunks[0]);
    draw_verses(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    if app.show_help {
        draw_help_overlay(f, app, area);
    }
}

// ── Header: day info (left) + chapter navigation (right) ──

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let w = area.width as usize;

    let day_label = format!(
        "  Day {} \u{00b7} {} \u{00b7} {}",
        app.day,
        app.date_string,
        app.translation.to_uppercase()
    );

    let mut nav_spans: Vec<Span> = Vec::new();
    for (i, ch) in app.today_chapters.iter().enumerate() {
        if i > 0 {
            nav_spans.push(Span::styled("  \u{00b7}  ", theme.header_separator_style()));
        }
        let label = format!("{} {}", ch.book, ch.chapter);
        if i == app.active_chapter_idx {
            nav_spans.push(Span::styled(label, theme.header_chapter_active()));
        } else {
            nav_spans.push(Span::styled(label, theme.header_chapter_inactive()));
        }
    }

    let day_width = day_label.chars().count();
    let nav_width: usize = nav_spans.iter().map(|s| s.content.chars().count()).sum();
    let right_pad = 2;
    let spacer = w.saturating_sub(day_width + nav_width + right_pad);

    let mut spans = vec![Span::styled(day_label, theme.header_day_style())];
    spans.push(Span::styled(" ".repeat(spacer), theme.bg_style()));
    spans.extend(nav_spans);
    spans.push(Span::styled(" ".repeat(right_pad), theme.bg_style()));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(vec![line]).style(theme.bg_style());
    f.render_widget(paragraph, area);
}

// ── Verse rendering with custom word wrapping ──

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.chars().count() + 1 + word.chars().count() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Build display lines for a single verse with hanging indent.
/// Format: "    NNN  text..."  (4-space margin, 3-char number, 2-space gap = 9-char prefix)
///         "         continuation..."
fn build_verse_lines(
    verse_num: u16,
    text: &str,
    content_width: u16,
    num_style: Style,
    text_style: Style,
) -> Vec<Line<'static>> {
    let prefix_len: usize = 9; // 4 margin + 3 number + 2 gap
    let text_width = (content_width as usize).saturating_sub(prefix_len);

    if text_width == 0 {
        return vec![Line::from(Span::styled(
            format!("{:>3}  {}", verse_num, text),
            text_style,
        ))];
    }

    let segments = word_wrap(text, text_width);
    let mut lines = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        let pad_count = text_width.saturating_sub(segment.chars().count());
        if i == 0 {
            let num_str = format!("    {:>3}  ", verse_num);
            lines.push(Line::from(vec![
                Span::styled(num_str, num_style),
                Span::styled(
                    format!("{}{}", segment, " ".repeat(pad_count)),
                    text_style,
                ),
            ]));
        } else {
            let indent = " ".repeat(prefix_len);
            lines.push(Line::from(vec![
                Span::styled(indent, text_style),
                Span::styled(
                    format!("{}{}", segment, " ".repeat(pad_count)),
                    text_style,
                ),
            ]));
        }
    }

    lines
}

/// Apply fade effect to a style's foreground color (lerp toward bg)
fn fade_style(style: Style, bg_color: Color, fade: f32) -> Style {
    if fade >= 1.0 {
        return style;
    }
    if let Some(fg) = style.fg {
        Style { fg: Some(lerp_color(bg_color, fg, fade)), ..style }
    } else {
        style
    }
}

/// Apply flash brightening to both foreground and background
fn flash_style(style: Style, intensity: f32) -> Style {
    if intensity <= 0.0 {
        return style;
    }
    let bright_fg = Color::Rgb(255, 255, 230);
    let bright_bg = Color::Rgb(80, 72, 40);
    let mut s = style;
    if let Some(fg) = s.fg {
        s.fg = Some(lerp_color(fg, bright_fg, intensity * 0.6));
    }
    if let Some(bg) = s.bg {
        s.bg = Some(lerp_color(bg, bright_bg, intensity * 0.7));
    }
    s
}

fn draw_verses(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme;

    let Some(ch_ref) = app.active_chapter() else {
        let msg = Paragraph::new("  No readings for today.").style(theme.bg_style());
        f.render_widget(msg, area);
        return;
    };
    let ch_book = ch_ref.book.clone();
    let ch_chapter = ch_ref.chapter;

    let verses = match app.active_verses() {
        Some(v) => v.clone(),
        None => {
            let msg = Paragraph::new(format!("  Chapter not found: {} {}", ch_book, ch_chapter))
                .style(theme.bg_style());
            f.render_widget(msg, area);
            return;
        }
    };

    // Center reading column (max 80 chars)
    let content_width: u16 = 80.min(area.width);
    let pad = (area.width.saturating_sub(content_width)) / 2;
    let content_area = Rect {
        x: area.x + pad,
        y: area.y,
        width: content_width,
        height: area.height,
    };

    let visual_range = if app.mode == Mode::Visual {
        Some(app.visual_range())
    } else {
        None
    };

    // Animation state
    let fade = app.anim.fade_progress();
    let bg_color = theme.bg();

    let w = content_width as usize;
    let mut all_lines: Vec<Line<'static>> = Vec::new();
    let mut verse_start: Vec<usize> = Vec::new();

    // Chapter title (centered, with fade)
    let title = format!("{} {}", ch_book, ch_chapter);
    let title_len = title.chars().count();
    let left_pad = w.saturating_sub(title_len) / 2;
    let right_pad = w.saturating_sub(left_pad + title_len);
    let title_style = fade_style(theme.chapter_title_style(), bg_color, fade);
    all_lines.push(Line::from(vec![
        Span::styled(" ".repeat(left_pad), theme.bg_style()),
        Span::styled(title, title_style),
        Span::styled(" ".repeat(right_pad), theme.bg_style()),
    ]));
    all_lines.push(Line::from(Span::styled(" ".repeat(w), theme.bg_style())));

    // Verses
    for (idx, verse) in verses.iter().enumerate() {
        verse_start.push(all_lines.len());

        let is_cursor = idx == app.cursor_verse;
        let is_highlighted =
            app.highlights
                .is_highlighted(&ch_book, ch_chapter, verse.number);
        let is_visual = visual_range.map_or(false, |(s, e)| idx >= s && idx <= e);

        let (mut num_style, mut text_style) =
            theme.verse_styles(is_cursor, is_highlighted, is_visual);

        // Chapter fade-in
        if fade < 1.0 {
            num_style = fade_style(num_style, bg_color, fade);
            text_style = fade_style(text_style, bg_color, fade);
        }

        // Highlight flash
        let flash = app.anim.flash_intensity(idx);
        if flash > 0.0 {
            num_style = flash_style(num_style, flash);
            text_style = flash_style(text_style, flash);
        }

        let verse_lines =
            build_verse_lines(verse.number, &verse.text, content_width, num_style, text_style);
        all_lines.extend(verse_lines);
    }

    // Scroll management: keep cursor verse visible
    let view_height = content_area.height as usize;
    if view_height > 0 && !verses.is_empty() {
        let cursor_start = verse_start
            .get(app.cursor_verse)
            .copied()
            .unwrap_or(0);
        let cursor_end = if app.cursor_verse + 1 < verse_start.len() {
            verse_start[app.cursor_verse + 1]
        } else {
            all_lines.len()
        };

        let scroll = app.scroll_offset as usize;

        if cursor_start < scroll {
            app.scroll_offset = cursor_start.saturating_sub(1) as u16;
        } else if cursor_end > scroll + view_height {
            app.scroll_offset = cursor_end.saturating_sub(view_height) as u16;
        }
    }

    // Render visible slice
    let scroll = app.scroll_offset as usize;
    let visible: Vec<Line<'static>> = all_lines
        .into_iter()
        .skip(scroll)
        .take(view_height)
        .collect();

    let paragraph = Paragraph::new(visible).style(theme.bg_style());
    f.render_widget(paragraph, content_area);
}

// ── Status bar ──

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let w = area.width as usize;

    let spans = match app.mode {
        Mode::Normal => {
            let left = if app.pending_g {
                "  g..".to_string()
            } else {
                "  ? help".to_string()
            };

            let right = if app.verse_count() > 0 {
                format!("verse {} of {}  ", app.cursor_verse + 1, app.verse_count())
            } else {
                String::new()
            };

            let spacer = w.saturating_sub(left.chars().count() + right.chars().count());

            vec![
                Span::styled(left, theme.status_style()),
                Span::styled(" ".repeat(spacer), theme.bg_style()),
                Span::styled(right, theme.status_style()),
            ]
        }
        Mode::Visual => {
            let (start, end) = app.visual_range();
            let count = end - start + 1;
            let left = format!(
                "  VISUAL  {} verse{}",
                count,
                if count == 1 { "" } else { "s" }
            );
            let right = "y:highlight  d:remove  Esc:cancel  ".to_string();
            let spacer = w.saturating_sub(left.chars().count() + right.chars().count());

            vec![
                Span::styled(left, theme.status_accent_style()),
                Span::styled(" ".repeat(spacer), theme.bg_style()),
                Span::styled(right, theme.status_style()),
            ]
        }
    };

    let line = Line::from(spans);
    let paragraph = Paragraph::new(vec![line]).style(theme.bg_style());
    f.render_widget(paragraph, area);
}

// ── Help overlay ──

fn draw_help_overlay(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;

    let popup_width = 52u16.min(area.width.saturating_sub(4));
    let popup_height = 19u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let key = theme.help_key_style();
    let desc = theme.help_desc_style();
    let title = theme.help_title_style();
    let bg = theme.help_bg_style();

    let help_lines = vec![
        Line::from(Span::styled("", bg)),
        Line::from(Span::styled("  Navigation", title)),
        Line::from(Span::styled("", bg)),
        Line::from(vec![
            Span::styled("    j / k       ", key),
            Span::styled("Move up / down", desc),
        ]),
        Line::from(vec![
            Span::styled("    h / l       ", key),
            Span::styled("Prev / next chapter", desc),
        ]),
        Line::from(vec![
            Span::styled("    gg / G      ", key),
            Span::styled("First / last verse", desc),
        ]),
        Line::from(vec![
            Span::styled("    L / H  ", key),
            Span::styled("Half-page down / up", desc),
        ]),
        Line::from(Span::styled("", bg)),
        Line::from(Span::styled("  Highlighting", title)),
        Line::from(Span::styled("", bg)),
        Line::from(vec![
            Span::styled("    Enter       ", key),
            Span::styled("Toggle highlight", desc),
        ]),
        Line::from(vec![
            Span::styled("    v           ", key),
            Span::styled("Visual selection", desc),
        ]),
        Line::from(vec![
            Span::styled("    y / d       ", key),
            Span::styled("Highlight / remove (visual)", desc),
        ]),
        Line::from(Span::styled("", bg)),
        Line::from(vec![
            Span::styled("    t           ", key),
            Span::styled("Cycle theme", desc),
        ]),
        Line::from(vec![
            Span::styled("    T           ", key),
            Span::styled("Cycle translation", desc),
        ]),
        Line::from(vec![
            Span::styled("    q           ", key),
            Span::styled("Quit", desc),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(" Keybindings ", theme.help_title_style()))
        .border_style(theme.help_border_style())
        .style(theme.help_bg_style());

    let paragraph = Paragraph::new(help_lines).block(block);
    f.render_widget(paragraph, popup_area);
}
