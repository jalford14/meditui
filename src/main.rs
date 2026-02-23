mod app;
mod bible;
mod highlight;
mod keys;
mod plan;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

const BIBLE_JSON: &str = include_str!("../data/bible_merged.json");

fn main() -> io::Result<()> {
    // Parse embedded data
    let bible = bible::Bible::from_json(BIBLE_JSON);
    let reading_plan = plan::Plan::new();
    let highlights = highlight::Highlights::load();

    let mut app = app::App::new(bible, reading_plan, highlights);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        let visible_lines = terminal.get_frame().area().height.saturating_sub(4) as usize;

        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                keys::handle_key(&mut app, key, visible_lines);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
