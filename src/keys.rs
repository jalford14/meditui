use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};

pub fn handle_key(app: &mut App, key: KeyEvent, visible_lines: usize) {
    if key.kind == crossterm::event::KeyEventKind::Release {
        return;
    }
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
        Mode::Highlights => handle_highlights(app, key),
        Mode::Notes => match key.code {
            KeyCode::Esc => app.mode = Mode::Normal,
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                app.notes_scroll = (app.notes_scroll + 1).min(app.notes_max_scroll)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.notes_scroll = app.notes_scroll.saturating_sub(1)
            }
            KeyCode::Char('?') => app.toggle_help(),
            _ => {}
        },
        Mode::NoteEditor => handle_note_editor(app, key),
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

        KeyCode::Char('L') => app.goto_visible_last(),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_down(visible_lines);
        }

        KeyCode::Char('H') => app.goto_visible_first(),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_up(visible_lines);
        }

        KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => app.next_chapter(),
        KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => app.prev_chapter(),

        KeyCode::Char('a') => app.enter_highlights(),
        KeyCode::Char('n') => app.begin_note(),
        KeyCode::Char('N') => app.enter_notes(),
        KeyCode::Char('r') => app.goto_today(),
        KeyCode::Char('v') => app.enter_visual(),
        KeyCode::Enter => app.toggle_highlight(),
        KeyCode::Char('t') => app.cycle_theme(),
        KeyCode::Char('T') => app.cycle_translation(),
        KeyCode::Char('?') => app.toggle_help(),

        _ => {}
    }
}

fn handle_highlights(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc => app.exit_highlights(),

        KeyCode::Char('j') | KeyCode::Down => app.highlight_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.highlight_cursor_up(),
        KeyCode::Enter => app.open_selected_highlight(),
        KeyCode::Char('n') => {
            if !app.highlight_items().is_empty() {
                app.open_selected_highlight();
                app.begin_note();
            }
        }
        KeyCode::Char('N') => app.enter_notes(),

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
        KeyCode::Char('n') => app.begin_note(),
        KeyCode::Char('N') => app.enter_notes(),

        KeyCode::Esc => app.cancel_visual(),
        KeyCode::Char('?') => app.toggle_help(),

        _ => {}
    }
}

fn handle_note_editor(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_note(),
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => app.save_note(),
        KeyCode::Backspace => {
            if let Some(note) = &mut app.note_draft {
                note.text.pop();
            }
        }
        KeyCode::Enter => {
            if let Some(note) = &mut app.note_draft {
                note.text.push('\n');
            }
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            if let Some(note) = &mut app.note_draft {
                note.text.push(c);
            }
        }
        _ => {}
    }
}
