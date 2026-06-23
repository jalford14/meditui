use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};

pub fn handle_key(app: &mut App, key: KeyEvent, visible_lines: usize) {
    // Any key dismisses help overlay
    if app.show_help {
        app.show_help = false;
        return;
    }

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

        KeyCode::Char('L') => {
            app.half_page_down(visible_lines);
        }
        KeyCode::Char('H') => {
            app.half_page_up(visible_lines);
        }

        KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => app.next_chapter(),
        KeyCode::Char('h') | KeyCode::Left  | KeyCode::BackTab => app.prev_chapter(),

        KeyCode::Char('v') => app.enter_visual(),
        KeyCode::Enter => app.toggle_highlight(),
        KeyCode::Char('t') => app.cycle_theme(),
        KeyCode::Char('T') => app.cycle_translation(),
        KeyCode::Char('?') => app.toggle_help(),

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
        KeyCode::Char('?') => app.toggle_help(),

        _ => {}
    }
}
