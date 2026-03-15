use std::fs;
use std::path::PathBuf;

use crate::animation::AnimationState;
use crate::bible::Bible;
use crate::highlight::Highlights;
use crate::plan::{ChapterRef, Plan};
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Visual,
}

pub struct App {
    pub bible: Bible,
    #[allow(dead_code)]
    pub plan: Plan,
    pub highlights: Highlights,
    pub today_chapters: Vec<ChapterRef>,
    pub active_chapter_idx: usize,
    pub cursor_verse: usize,
    pub scroll_offset: u16,
    pub mode: Mode,
    pub visual_anchor: usize,
    pub pending_g: bool,
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
        data_dir: PathBuf,
        translation: String,
        available_translations: Vec<String>,
    ) -> App {
        let day = crate::plan::day_of_year();
        let date_string = crate::plan::today_date_string();
        let today_chapters = plan.chapters_for_day(day);
        let theme = Theme::load();

        App {
            bible,
            plan,
            highlights,
            today_chapters,
            active_chapter_idx: 0,
            cursor_verse: 0,
            scroll_offset: 0,
            mode: Mode::Normal,
            visual_anchor: 0,
            pending_g: false,
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
        self.mode = Mode::Normal;
        self.anim.start_fade();
    }

    pub fn active_chapter(&self) -> Option<&ChapterRef> {
        self.today_chapters.get(self.active_chapter_idx)
    }

    pub fn active_verses(&self) -> Option<&Vec<crate::bible::Verse>> {
        self.active_chapter()
            .and_then(|ch| self.bible.get_chapter(&ch.book, ch.chapter))
    }

    pub fn verse_count(&self) -> usize {
        self.active_verses().map_or(0, |v| v.len())
    }

    pub fn cursor_down(&mut self) {
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

    pub fn half_page_down(&mut self, visible_lines: usize) {
        let half = visible_lines / 2;
        let count = self.verse_count();
        if count > 0 {
            self.cursor_verse = (self.cursor_verse + half).min(count - 1);
        }
    }

    pub fn half_page_up(&mut self, visible_lines: usize) {
        let half = visible_lines / 2;
        self.cursor_verse = self.cursor_verse.saturating_sub(half);
    }

    pub fn next_chapter(&mut self) {
        if self.active_chapter_idx + 1 < self.today_chapters.len() {
            self.active_chapter_idx += 1;
            self.cursor_verse = 0;
            self.scroll_offset = 0;
            self.mode = Mode::Normal;
            self.anim.start_fade();
        }
    }

    pub fn prev_chapter(&mut self) {
        if self.active_chapter_idx > 0 {
            self.active_chapter_idx -= 1;
            self.cursor_verse = 0;
            self.scroll_offset = 0;
            self.mode = Mode::Normal;
            self.anim.start_fade();
        }
    }

    pub fn enter_visual(&mut self) {
        self.mode = Mode::Visual;
        self.visual_anchor = self.cursor_verse;
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
                let verse_nums: Vec<u16> = verses[start..=end]
                    .iter()
                    .map(|v| v.number)
                    .collect();
                self.highlights.highlight_range(&ch.book, ch.chapter, &verse_nums);
                self.highlights.save();
            }
        }
        self.mode = Mode::Normal;
    }

    pub fn unhighlight_selection(&mut self) {
        if let Some(ch) = self.active_chapter().cloned() {
            if let Some(verses) = self.active_verses() {
                let (start, end) = self.visual_range();
                let verse_nums: Vec<u16> = verses[start..=end]
                    .iter()
                    .map(|v| v.number)
                    .collect();
                self.highlights.unhighlight_range(&ch.book, ch.chapter, &verse_nums);
                self.highlights.save();
            }
        }
        self.mode = Mode::Normal;
    }

    pub fn toggle_highlight(&mut self) {
        if let Some(ch) = self.active_chapter().cloned() {
            if let Some(verses) = self.active_verses() {
                if let Some(verse) = verses.get(self.cursor_verse) {
                    self.highlights.toggle(&ch.book, ch.chapter, verse.number);
                    self.highlights.save();
                    self.anim.start_flash(self.cursor_verse);
                }
            }
        }
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
            if t.is_empty() { "kjv".to_string() } else { t }
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
