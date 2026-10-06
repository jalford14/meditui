use cli_log::info;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::animation::AnimationState;
use crate::bible::Bible;
use crate::highlight::Highlights;
use crate::notes::{Note, Notes};
use crate::plan::{ChapterRef, Plan};
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Visual,
    Highlights,
    Notes,
    NoteEditor,
}

#[derive(Clone, Debug)]
pub struct HighlightItem {
    pub day: u16,
    pub book: String,
    pub chapter: u16,
    pub verses: Vec<u16>,
}

pub struct App {
    pub bible: Bible,
    pub plan: Plan,
    pub highlights: Highlights,
    pub notes: Notes,
    pub note_draft: Option<Note>,
    pub note_error: Option<String>,
    pub notes_scroll: usize,
    pub notes_max_scroll: usize,
    pub note_return_mode: Mode,
    pub today_chapters: Vec<ChapterRef>,
    pub active_chapter_idx: usize,
    pub cursor_verse: usize,
    pub scroll_offset: u16,
    pub visible_verse_range: Option<(usize, usize)>,
    pub mode: Mode,
    pub visual_anchor: usize,
    pub pending_g: bool,
    pub highlight_cursor: usize,
    pub highlight_scroll_offset: u16,
    pub day: u16,
    pub date_string: String,
    pub should_quit: bool,
    pub theme: Theme,
    pub show_help: bool,
    pub anim: AnimationState,
    pub data_dir: PathBuf,
    pub translation: String,
    pub available_translations: Vec<String>,
}

impl App {
    pub fn new(
        bible: Bible,
        plan: Plan,
        highlights: Highlights,
        notes: Notes,
        data_dir: PathBuf,
        translation: String,
        available_translations: Vec<String>,
    ) -> App {
        let day = crate::plan::day_of_year();
        let date_string = crate::plan::date_string_for_day(day);
        let today_chapters = plan.chapters_for_day(day);
        let theme = Theme::load();

        App {
            bible,
            plan,
            highlights,
            notes,
            note_draft: None,
            note_error: None,
            notes_scroll: 0,
            notes_max_scroll: 0,
            note_return_mode: Mode::Normal,
            today_chapters,
            active_chapter_idx: 0,
            cursor_verse: 0,
            scroll_offset: 0,
            visible_verse_range: None,
            mode: Mode::Normal,
            visual_anchor: 0,
            pending_g: false,
            highlight_cursor: 0,
            highlight_scroll_offset: 0,
            day,
            date_string,
            should_quit: false,
            theme,
            show_help: false,
            anim: AnimationState::new(),
            data_dir,
            translation,
            available_translations,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.cycle();
        self.theme.save();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn cycle_translation(&mut self) {
        if self.available_translations.len() <= 1 {
            return;
        }
        let idx = self
            .available_translations
            .iter()
            .position(|t| t == &self.translation)
            .unwrap_or(0);
        let next = (idx + 1) % self.available_translations.len();
        self.translation = self.available_translations[next].clone();
        save_translation(&self.translation);

        // Reload bible data for the new translation
        let chapter_refs: Vec<(String, u16)> = self
            .today_chapters
            .iter()
            .map(|ch| (ch.book.clone(), ch.chapter))
            .collect();
        self.bible = Bible::load_chapters(&self.data_dir, &self.translation, &chapter_refs);

        // Reset view state
        self.cursor_verse = 0;
        self.scroll_offset = 0;
        self.visible_verse_range = None;
        self.mode = Mode::Normal;
        self.anim.start_fade();
    }

    pub fn active_chapter(&self) -> Option<&ChapterRef> {
        self.today_chapters.get(self.active_chapter_idx)
    }

    pub fn active_verses(&self) -> Option<Vec<crate::bible::Verse>> {
        self.active_chapter().and_then(|ch| {
            let all = self.bible.get_chapter(&ch.book, ch.chapter)?;
            let filtered: Vec<crate::bible::Verse> = all
                .iter()
                .filter(|v| {
                    ch.verse_start.map_or(true, |s| v.number >= s)
                        && ch.verse_end.map_or(true, |e| v.number <= e)
                })
                .cloned()
                .collect();
            Some(filtered)
        })
    }

    pub fn verse_count(&self) -> usize {
        self.active_verses().map_or(0, |v| v.len())
    }

    pub fn cursor_down(&mut self) {
        info!("verse_count: {}", self.verse_count());
        info!("cursor_verse: {}", self.cursor_verse);
        let count = self.verse_count();
        if count > 0 && self.cursor_verse < count - 1 {
            self.cursor_verse += 1;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor_verse > 0 {
            self.cursor_verse -= 1;
        }
    }

    pub fn goto_first(&mut self) {
        self.cursor_verse = 0;
    }

    pub fn goto_last(&mut self) {
        let count = self.verse_count();
        if count > 0 {
            self.cursor_verse = count - 1;
        }
    }

    pub fn set_visible_verse_range(&mut self, range: Option<(usize, usize)>) {
        self.visible_verse_range = range;
    }

    pub fn goto_visible_first(&mut self) {
        if let Some((first, _)) = self.visible_verse_range {
            self.cursor_verse = first;
        }
    }

    pub fn goto_visible_last(&mut self) {
        if let Some((_, last)) = self.visible_verse_range {
            self.cursor_verse = last;
        }
    }

    pub fn half_page_down(&mut self, visible_lines: usize) {
        let half = self
            .visible_verse_range
            .map(|(first, last)| ((last - first + 1) / 2).max(1))
            .unwrap_or_else(|| (visible_lines / 2).max(1));
        let count = self.verse_count();
        if count > 0 {
            self.cursor_verse = (self.cursor_verse + half).min(count - 1);
        }
    }

    pub fn half_page_up(&mut self, visible_lines: usize) {
        let half = self
            .visible_verse_range
            .map(|(first, last)| ((last - first + 1) / 2).max(1))
            .unwrap_or_else(|| (visible_lines / 2).max(1));
        self.cursor_verse = self.cursor_verse.saturating_sub(half);
    }

    pub fn load_day(&mut self, day: u16) {
        let day = day.clamp(1, crate::plan::DAY_COUNT);
        let chapters = self.plan.chapters_for_day(day);
        let chapter_refs: Vec<(String, u16)> = chapters
            .iter()
            .map(|ch| (ch.book.clone(), ch.chapter))
            .collect();

        self.bible = Bible::load_chapters(&self.data_dir, &self.translation, &chapter_refs);
        self.today_chapters = chapters;
        self.active_chapter_idx = 0;
        self.cursor_verse = 0;
        self.scroll_offset = 0;
        self.visible_verse_range = None;
        self.mode = Mode::Normal;
        self.visual_anchor = 0;
        self.pending_g = false;
        self.day = day;
        self.date_string = crate::plan::date_string_for_day(day);
        self.anim.start_fade();
    }

    pub fn goto_today(&mut self) {
        self.load_day(crate::plan::day_of_year().min(crate::plan::DAY_COUNT));
    }

    pub fn next_chapter(&mut self) {
        if self.active_chapter_idx + 1 < self.today_chapters.len() {
            self.active_chapter_idx += 1;
            self.cursor_verse = 0;
            self.scroll_offset = 0;
            self.visible_verse_range = None;
            self.mode = Mode::Normal;
            self.anim.start_fade();
        }
    }

    pub fn prev_chapter(&mut self) {
        if self.active_chapter_idx > 0 {
            self.active_chapter_idx -= 1;
            self.cursor_verse = 0;
            self.scroll_offset = 0;
            self.visible_verse_range = None;
            self.mode = Mode::Normal;
            self.anim.start_fade();
        }
    }

    pub fn enter_visual(&mut self) {
        self.mode = Mode::Visual;
        self.visual_anchor = self.cursor_verse;
    }

    pub fn enter_notes(&mut self) {
        self.mode = Mode::Notes;
        self.pending_g = false;
    }

    pub fn begin_note(&mut self) {
        let Some(ch) = self.active_chapter().cloned() else {
            return;
        };
        let Some(verses) = self.active_verses() else {
            return;
        };
        let (start, end) = if self.mode == Mode::Visual {
            self.visual_range()
        } else {
            (self.cursor_verse, self.cursor_verse)
        };
        let Some(selected) = verses.get(start..=end) else {
            return;
        };
        if selected.is_empty() {
            return;
        }
        self.note_draft = Some(Note {
            book: ch.book,
            chapter: ch.chapter,
            verses: selected.iter().map(|verse| verse.number).collect(),
            date: chrono::Local::now().format("%d/%m/%Y").to_string(),
            text: String::new(),
        });
        self.note_error = None;
        self.note_return_mode = self.mode;
        self.mode = Mode::NoteEditor;
    }

    pub fn cancel_note(&mut self) {
        self.note_draft = None;
        self.note_error = None;
        self.mode = self.note_return_mode;
    }

    pub fn save_note(&mut self) {
        let Some(mut note) = self.note_draft.clone() else {
            return;
        };
        note.text = note.text.trim().to_string();
        if note.text.is_empty() {
            self.note_error = Some("Write a note before saving.".to_string());
            return;
        }
        self.notes.entries.push(note.clone());
        if let Err(error) = self.notes.save() {
            self.notes.entries.pop();
            self.note_error = Some(format!("Could not save note: {error}"));
            return;
        }
        self.highlights
            .highlight_range_for_day(&note.book, note.chapter, &note.verses, self.day);
        self.highlights.save();
        self.note_draft = None;
        self.note_error = None;
        self.mode = Mode::Normal;
    }

    pub fn cancel_visual(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn visual_range(&self) -> (usize, usize) {
        let a = self.visual_anchor;
        let b = self.cursor_verse;
        (a.min(b), a.max(b))
    }

    pub fn highlight_selection(&mut self) {
        if let Some(ch) = self.active_chapter().cloned() {
            if let Some(verses) = self.active_verses() {
                let (start, end) = self.visual_range();
                let verse_nums: Vec<u16> = verses[start..=end].iter().map(|v| v.number).collect();
                self.highlights.highlight_range_for_day(
                    &ch.book,
                    ch.chapter,
                    &verse_nums,
                    self.day,
                );
                self.highlights.save();
            }
        }
        self.mode = Mode::Normal;
    }

    pub fn unhighlight_selection(&mut self) {
        if let Some(ch) = self.active_chapter().cloned() {
            if let Some(verses) = self.active_verses() {
                let (start, end) = self.visual_range();
                let verse_nums: Vec<u16> = verses[start..=end].iter().map(|v| v.number).collect();
                self.highlights
                    .unhighlight_range(&ch.book, ch.chapter, &verse_nums);
                self.highlights.save();
            }
        }
        self.mode = Mode::Normal;
    }

    pub fn toggle_highlight(&mut self) {
        if let Some(ch) = self.active_chapter().cloned() {
            if let Some(verses) = self.active_verses() {
                if let Some(verse) = verses.get(self.cursor_verse) {
                    self.highlights
                        .toggle_for_day(&ch.book, ch.chapter, verse.number, self.day);
                    self.highlights.save();
                    self.anim.start_flash(self.cursor_verse);
                }
            }
        }
    }

    pub fn enter_highlights(&mut self) {
        self.mode = Mode::Highlights;
        self.pending_g = false;
        self.clamp_highlight_cursor();
    }

    pub fn exit_highlights(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn highlight_cursor_down(&mut self) {
        let count = self.highlight_items().len();
        if count > 0 && self.highlight_cursor + 1 < count {
            self.highlight_cursor += 1;
        }
    }

    pub fn highlight_cursor_up(&mut self) {
        if self.highlight_cursor > 0 {
            self.highlight_cursor -= 1;
        }
    }

    pub fn open_selected_highlight(&mut self) {
        let Some(item) = self.highlight_items().get(self.highlight_cursor).cloned() else {
            return;
        };

        self.load_day(item.day);

        if let Some(chapter_idx) = self
            .today_chapters
            .iter()
            .position(|ch| ch.book == item.book && ch.chapter == item.chapter)
        {
            self.active_chapter_idx = chapter_idx;
        }

        let cursor = self.active_verses().and_then(|verses| {
            item.verses
                .first()
                .and_then(|first| verses.iter().position(|verse| verse.number == *first))
        });

        if let Some(cursor) = cursor {
            self.cursor_verse = cursor;
        }
        self.scroll_offset = 0;
        self.visible_verse_range = None;
    }

    pub fn clamp_highlight_cursor(&mut self) {
        let count = self.highlight_items().len();
        if count == 0 {
            self.highlight_cursor = 0;
            self.highlight_scroll_offset = 0;
        } else if self.highlight_cursor >= count {
            self.highlight_cursor = count - 1;
        }
    }

    pub fn highlight_items(&self) -> Vec<HighlightItem> {
        let mut by_day: BTreeMap<u16, BTreeMap<(String, u16), BTreeSet<u16>>> = BTreeMap::new();
        let mut explicit_highlights: HashSet<(String, u16, u16)> = HashSet::new();

        for (&day, chapters) in &self.highlights.days {
            if !(1..=crate::plan::DAY_COUNT).contains(&day) {
                continue;
            }

            for (key, verses) in chapters {
                let Some((book, chapter)) = Highlights::parse_key(key) else {
                    continue;
                };

                for &verse in verses {
                    if self.highlights.is_highlighted(&book, chapter, verse) {
                        by_day
                            .entry(day)
                            .or_default()
                            .entry((book.clone(), chapter))
                            .or_default()
                            .insert(verse);
                        explicit_highlights.insert((book.clone(), chapter, verse));
                    }
                }
            }
        }

        for day in 1..=crate::plan::DAY_COUNT {
            for chapter in self.plan.chapters_for_day(day) {
                let Some(verses) = self
                    .highlights
                    .highlighted_verses(&chapter.book, chapter.chapter)
                else {
                    continue;
                };

                for &verse in verses {
                    let key = (chapter.book.clone(), chapter.chapter, verse);
                    if explicit_highlights.contains(&key) {
                        continue;
                    }

                    by_day
                        .entry(day)
                        .or_default()
                        .entry((chapter.book.clone(), chapter.chapter))
                        .or_default()
                        .insert(verse);
                }
            }
        }

        by_day
            .into_iter()
            .flat_map(|(day, chapters)| {
                chapters
                    .into_iter()
                    .map(move |((book, chapter), verses)| HighlightItem {
                        day,
                        book,
                        chapter,
                        verses: verses.into_iter().collect(),
                    })
            })
            .collect()
    }
}

// -- Translation config persistence --

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("bible-tui").join("translation"))
}

pub fn load_translation() -> String {
    let Some(path) = config_path() else {
        return "kjv".to_string();
    };
    match fs::read_to_string(path) {
        Ok(s) => {
            let t = s.trim().to_string();
            if t.is_empty() {
                "kjv".to_string()
            } else {
                t
            }
        }
        Err(_) => "kjv".to_string(),
    }
}

fn save_translation(translation: &str) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, translation);
}

/// Discover available translations by scanning subdirectories of data_dir.
pub fn discover_translations(data_dir: &std::path::Path) -> Vec<String> {
    let mut translations = Vec::new();
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    translations.push(name.to_string());
                }
            }
        }
    }
    translations.sort();
    translations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn dated_notes_selection_persistence_and_failed_save() {
        let directory = std::env::temp_dir().join(format!(
            "meditui-notes-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap()
        ));
        let path = directory.join("notes.json");
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let mut app = App::new(
            Bible::load_chapters(&data_dir, "kjv", &[]),
            Plan::new(),
            Highlights::default(),
            Notes::load_from(path.clone()).unwrap(),
            data_dir,
            "kjv".to_string(),
            vec!["kjv".to_string()],
        );
        let day = (1..=crate::plan::DAY_COUNT)
            .find(|&day| {
                app.plan
                    .chapters_for_day(day)
                    .iter()
                    .any(|ch| ch.book == "Luke" && ch.verse_start == Some(39))
            })
            .unwrap();
        app.load_day(day); // Luke 1:39-80: indexes differ from verse numbers.
        app.active_chapter_idx = app
            .today_chapters
            .iter()
            .position(|ch| ch.book == "Luke")
            .unwrap();
        let key = |code, modifiers| KeyEvent::new(code, modifiers);
        crate::keys::handle_key(&mut app, key(KeyCode::Char('v'), KeyModifiers::NONE), 20);
        app.cursor_down();
        crate::keys::handle_key(&mut app, key(KeyCode::Char('n'), KeyModifiers::NONE), 20);
        assert_eq!(app.note_draft.as_ref().unwrap().verses, vec![39, 40]);
        assert_eq!(
            app.note_draft.as_ref().unwrap().date,
            chrono::Local::now().format("%d/%m/%Y").to_string()
        );
        app.save_note();
        assert!(app.note_error.is_some());
        assert!(app.notes.entries.is_empty());
        for c in "Reflection: café? q".chars() {
            crate::keys::handle_key(&mut app, key(KeyCode::Char(c), KeyModifiers::NONE), 20);
        }
        assert!(!app.should_quit);
        crate::keys::handle_key(&mut app, key(KeyCode::Enter, KeyModifiers::NONE), 20);
        crate::keys::handle_key(&mut app, key(KeyCode::Char('é'), KeyModifiers::NONE), 20);
        crate::keys::handle_key(&mut app, key(KeyCode::Backspace, KeyModifiers::NONE), 20);
        let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
        terminal
            .draw(|frame| crate::ui::draw(frame, &mut app))
            .unwrap();
        // Simulate an unwritable destination without touching the real config.
        fs::create_dir_all(&directory).unwrap();
        fs::create_dir(&path).unwrap();
        app.save_note();
        assert!(app.mode == Mode::NoteEditor);
        assert!(app.note_error.as_ref().unwrap().contains("Could not save"));
        assert!(app.notes.entries.is_empty());
        assert!(app.note_draft.as_ref().unwrap().text.contains("café"));
        fs::remove_dir(&path).unwrap();
        // Save notes directly here so the test never writes real highlight config.
        let note = app.note_draft.clone().unwrap();
        app.notes.entries.push(note);
        app.notes.save().unwrap();
        let loaded = Notes::load_from(path.clone()).unwrap();
        assert_eq!(loaded.entries[0].verses, vec![39, 40]);
        assert_eq!(loaded.entries[0].text, "Reflection: café? q\n");
        app.cancel_note();
        assert!(app.mode == Mode::Visual);
        app.enter_notes();
        terminal
            .draw(|frame| crate::ui::draw(frame, &mut app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Luke 1:39-40"));
        assert!(rendered.contains(&loaded.entries[0].date));
        assert!(rendered.contains("Reflection: café? q"));
        fs::write(&path, "invalid JSON").unwrap();
        assert!(Notes::load_from(path).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
