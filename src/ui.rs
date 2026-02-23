use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::vertical([
        Constraint::Length(1), // top bar
        Constraint::Length(2), // chapter tabs
        Constraint::Min(1),   // verses
        Constraint::Length(1), // status bar
        Constraint::Length(1), // help bar
    ])
    .split(area);

    draw_top_bar(f, app, chunks[0]);
    draw_chapter_tabs(f, app, chunks[1]);
    draw_verses(f, app, chunks[2]);
    draw_status_bar(f, app, chunks[3]);
    draw_help_bar(f, app, chunks[4]);
}

fn draw_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = format!(
        "  McHeyne Day {} — {}   Family | Secret",
        app.day, app.date_string
    );
    let bar = Paragraph::new(text).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(bar, area);
}

fn draw_chapter_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .today_chapters
        .iter()
        .map(|ch| Line::from(format!("{} {}", ch.book, ch.chapter)))
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_chapter_idx)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|")
        .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(tabs, area);
}

/// Estimate how many display lines a text string occupies at a given width.
fn wrapped_line_count(text: &str, width: u16) -> usize {
    if width == 0 {
        return 1;
    }
    let w = width as usize;
    let char_count = text.chars().count();
    if char_count <= w {
        1
    } else {
        // Word wrapping wastes some space at line breaks, so add a small margin
        let effective_width = w.max(1);
        (char_count + effective_width - 1) / effective_width
    }
}

fn draw_verses(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(ch_ref) = app.active_chapter() else {
        let msg = Paragraph::new("No readings for today.");
        f.render_widget(msg, area);
        return;
    };
    let ch_book = ch_ref.book.clone();
    let ch_chapter = ch_ref.chapter;

    let verses = match app.active_verses() {
        Some(v) => v.clone(),
        None => {
            let msg = Paragraph::new(format!("Chapter not found: {} {}", ch_book, ch_chapter));
            f.render_widget(msg, area);
            return;
        }
    };

    // Center the reading column
    let content_width: u16 = 80.min(area.width);
    let pad = (area.width.saturating_sub(content_width)) / 2;
    let content_area = Rect {
        x: area.x + pad,
        y: area.y,
        width: content_width,
        height: area.height,
    };

    let visual_range = if app.mode == Mode::Visual {
        let (s, e) = app.visual_range();
        Some((s, e))
    } else {
        None
    };

    // Calculate display line positions for each verse (accounting for word wrap)
    let header_lines: usize = 2; // title + blank line
    let mut verse_display_lines: Vec<usize> = Vec::with_capacity(verses.len());
    let mut cumulative = header_lines;

    for verse in verses.iter() {
        verse_display_lines.push(cumulative);
        let text = format!(" {:>3}  {}", verse.number, verse.text);
        cumulative += wrapped_line_count(&text, content_width);
    }

    // Calculate scroll to keep cursor visible
    let view_height = content_area.height as usize;
    if view_height > 0 && !verses.is_empty() {
        let cursor_start = verse_display_lines
            .get(app.cursor_verse)
            .copied()
            .unwrap_or(0);
        let cursor_verse_text = verses
            .get(app.cursor_verse)
            .map(|v| format!(">{:>3}  {}", v.number, v.text))
            .unwrap_or_default();
        let cursor_height = wrapped_line_count(&cursor_verse_text, content_width);
        let cursor_end = cursor_start + cursor_height;

        let scroll = app.scroll_offset as usize;

        // If cursor is above visible area, scroll up
        if cursor_start < scroll {
            app.scroll_offset = cursor_start.saturating_sub(1) as u16;
        }
        // If cursor is below visible area, scroll down
        else if cursor_end > scroll + view_height {
            app.scroll_offset = (cursor_end.saturating_sub(view_height) + 1) as u16;
        }
    }

    // Build lines
    let mut lines: Vec<Line> = Vec::new();

    // Chapter title
    lines.push(Line::from(Span::styled(
        format!("{} {}", ch_book, ch_chapter),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (idx, verse) in verses.iter().enumerate() {
        let is_cursor = idx == app.cursor_verse;
        let is_highlighted =
            app.highlights
                .is_highlighted(&ch_book, ch_chapter, verse.number);
        let is_visual = visual_range.map_or(false, |(s, e)| idx >= s && idx <= e);

        let indicator = if is_cursor { ">" } else { " " };
        let verse_text = format!("{}{:>3}  {}", indicator, verse.number, verse.text);

        let style = if is_visual && is_cursor {
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else if is_visual {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else if is_cursor && is_highlighted {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_cursor {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if is_highlighted {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(Span::styled(verse_text, style)));
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, content_area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Visual => "VISUAL",
    };

    let ch_info = if let Some(ch_ref) = app.active_chapter() {
        format!(
            "{} {} ({} verses)",
            ch_ref.book,
            ch_ref.chapter,
            app.verse_count()
        )
    } else {
        String::new()
    };

    let pending = if app.pending_g { " g.." } else { "" };

    let text = format!(" {} | {}{}", mode_str, ch_info, pending);
    let bar = Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::DarkGray));
    f.render_widget(bar, area);
}

fn draw_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.mode {
        Mode::Normal => " j/k:move  h/l:chapter  v:select  Enter:highlight  gg/G:top/end  Ctrl-d/u:page  q:quit",
        Mode::Visual => " j/k:extend  y:highlight  d:remove  Esc:cancel",
    };

    let bar = Paragraph::new(help).style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    f.render_widget(bar, area);
}
