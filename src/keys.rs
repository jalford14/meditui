use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};

pub fn handle_key(app: &mut App, key: KeyEvent, visible_lines: usize) {
    // Handle pending g first
    if app.pending_g {
        app.pending_g = false;
        if key.code == KeyCode::Char('g') {
            app.goto_first();
        }
        return;
    }

    match app.mode {
        Mode::Normal => handle_normal(app, key, visible_lines),
        Mode::Visual => handle_visual(app, key),
    }
}

fn handle_normal(app: &mut App, key: KeyEvent, visible_lines: usize) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,

        KeyCode::Char('j') | KeyCode::Down => app.cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.cursor_up(),

        KeyCode::Char('g') => {
            app.pending_g = true;
        }
        KeyCode::Char('G') => app.goto_last(),

        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_down(visible_lines);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_up(visible_lines);
        }

        KeyCode::Char('l') | KeyCode::Tab => app.next_chapter(),
        KeyCode::Char('h') | KeyCode::BackTab => app.prev_chapter(),

        KeyCode::Char('v') => app.enter_visual(),
        KeyCode::Enter => app.toggle_highlight(),

        _ => {}
    }
}

fn handle_visual(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.cursor_up(),

        KeyCode::Char('y') => app.highlight_selection(),
        KeyCode::Char('d') => app.unhighlight_selection(),

        KeyCode::Esc => app.cancel_visual(),

        _ => {}
    }
}
