use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Highlights {
    pub chapters: HashMap<String, HashSet<u16>>,
}

impl Highlights {
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .expect("Could not find config directory")
            .join("bible-tui");
        fs::create_dir_all(&config_dir).ok();
        config_dir.join("highlights.json")
    }

    pub fn load() -> Highlights {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Highlights::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(path, json).ok();
        }
    }

    fn key(book: &str, chapter: u16) -> String {
        format!("{}:{}", book, chapter)
    }

    pub fn is_highlighted(&self, book: &str, chapter: u16, verse: u16) -> bool {
        let key = Self::key(book, chapter);
        self.chapters
            .get(&key)
            .map_or(false, |vs| vs.contains(&verse))
    }

    pub fn toggle(&mut self, book: &str, chapter: u16, verse: u16) {
        let key = Self::key(book, chapter);
        let verses = self.chapters.entry(key).or_default();
        if !verses.remove(&verse) {
            verses.insert(verse);
        }
    }

    pub fn highlight_range(&mut self, book: &str, chapter: u16, verses: &[u16]) {
        let key = Self::key(book, chapter);
        let set = self.chapters.entry(key).or_default();
        for &v in verses {
            set.insert(v);
        }
    }

    pub fn unhighlight_range(&mut self, book: &str, chapter: u16, verses: &[u16]) {
        let key = Self::key(book, chapter);
        if let Some(set) = self.chapters.get_mut(&key) {
            for &v in verses {
                set.remove(&v);
            }
            if set.is_empty() {
                self.chapters.remove(&key);
            }
        }
    }
}
