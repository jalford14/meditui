mod animation;
mod app;
mod bible;
mod highlight;
mod keys;
mod plan;
mod theme;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use cli_log::*;

use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

fn find_data_dir() -> PathBuf {
    // Try relative to CWD first (normal usage: running from project root)
    let cwd = PathBuf::from("data");
    if cwd.exists() {
        return cwd;
    }

    // Try relative to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_data = parent.join("data");
            if exe_data.exists() {
                return exe_data;
            }
        }
    }

    // Fall back to compile-time path
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data"))
}

fn main() -> io::Result<()> {
    init_cli_log!();
    let data_dir = find_data_dir();
    let reading_plan = plan::Plan::new();
    let highlights = highlight::Highlights::load();

    // Discover available translations
    let available = app::discover_translations(&data_dir);
    let mut translation = app::load_translation();

    // Validate that the saved translation still exists
    if !available.contains(&translation) {
        translation = "kjv".to_string();
    }

    // Load only today's chapters
    let day = plan::day_of_year();
    let today_chapters = reading_plan.chapters_for_day(day);
    let chapter_refs: Vec<(String, u16)> = today_chapters
        .iter()
        .map(|ch| (ch.book.clone(), ch.chapter))
        .collect();
    let bible = bible::Bible::load_chapters(&data_dir, &translation, &chapter_refs);

    let mut app = app::App::new(bible, reading_plan, highlights, data_dir, translation, available);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        let visible_lines = terminal.get_frame().area().height.saturating_sub(2) as usize;

        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Faster polling during animations for smooth rendering (~30fps)
        let poll_ms = if app.anim.is_animating() { 5 } else { 100 };

        if event::poll(Duration::from_millis(poll_ms))? {
            if let Event::Key(key) = event::read()? {
                keys::handle_key(&mut app, key, visible_lines);
            }
        }

        app.anim.tick();

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
