use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Highlights {
    #[serde(default)]
    pub chapters: HashMap<String, HashSet<u16>>,
    #[serde(default)]
    pub days: HashMap<u16, HashMap<String, HashSet<u16>>>,
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

    pub fn key(book: &str, chapter: u16) -> String {
        format!("{}:{}", book, chapter)
    }

    pub fn parse_key(key: &str) -> Option<(String, u16)> {
        let (book, chapter) = key.rsplit_once(':')?;
        Some((book.to_string(), chapter.parse().ok()?))
    }

    pub fn is_highlighted(&self, book: &str, chapter: u16, verse: u16) -> bool {
        let key = Self::key(book, chapter);
        self.chapters
            .get(&key)
            .map_or(false, |vs| vs.contains(&verse))
    }

    pub fn highlighted_verses(&self, book: &str, chapter: u16) -> Option<&HashSet<u16>> {
        let key = Self::key(book, chapter);
        self.chapters.get(&key)
    }

    pub fn toggle_for_day(&mut self, book: &str, chapter: u16, verse: u16, day: u16) {
        let key = Self::key(book, chapter);
        let removed = {
            let verses = self.chapters.entry(key).or_default();
            if verses.remove(&verse) {
                true
            } else {
                verses.insert(verse);
                false
            }
        };

        if !removed {
            self.add_day_highlight(book, chapter, verse, day);
        } else {
            self.remove_from_days(book, chapter, verse);
        }

        self.remove_empty_chapter(book, chapter);
    }

    pub fn highlight_range_for_day(&mut self, book: &str, chapter: u16, verses: &[u16], day: u16) {
        let key = Self::key(book, chapter);
        {
            let set = self.chapters.entry(key).or_default();
            for &v in verses {
                set.insert(v);
            }
        }

        for &v in verses {
            self.add_day_highlight(book, chapter, v, day);
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

        for &v in verses {
            self.remove_from_days(book, chapter, v);
        }
    }

    fn add_day_highlight(&mut self, book: &str, chapter: u16, verse: u16, day: u16) {
        let key = Self::key(book, chapter);
        self.days
            .entry(day)
            .or_default()
            .entry(key)
            .or_default()
            .insert(verse);
    }

    fn remove_from_days(&mut self, book: &str, chapter: u16, verse: u16) {
        let key = Self::key(book, chapter);
        let mut empty_days = Vec::new();

        for (&day, chapters) in self.days.iter_mut() {
            if let Some(verses) = chapters.get_mut(&key) {
                verses.remove(&verse);
                if verses.is_empty() {
                    chapters.remove(&key);
                }
            }

            if chapters.is_empty() {
                empty_days.push(day);
            }
        }

        for day in empty_days {
            self.days.remove(&day);
        }
    }

    fn remove_empty_chapter(&mut self, book: &str, chapter: u16) {
        let key = Self::key(book, chapter);
        if self.chapters.get(&key).map_or(false, HashSet::is_empty) {
            self.chapters.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Highlights;

    #[test]
    fn loads_legacy_chapter_only_highlights() {
        let highlights: Highlights =
            serde_json::from_str(r#"{"chapters":{"Genesis:1":[1,3]}}"#).unwrap();

        assert!(highlights.is_highlighted("Genesis", 1, 1));
        assert!(highlights.is_highlighted("Genesis", 1, 3));
        assert!(highlights.days.is_empty());
    }

    #[test]
    fn toggles_highlight_in_global_and_day_indexes() {
        let mut highlights = Highlights::default();

        highlights.toggle_for_day("Genesis", 1, 2, 1);
        assert!(highlights.is_highlighted("Genesis", 1, 2));
        assert!(highlights.days[&1][&Highlights::key("Genesis", 1)].contains(&2));

        highlights.toggle_for_day("Genesis", 1, 2, 1);
        assert!(!highlights.is_highlighted("Genesis", 1, 2));
        assert!(highlights.chapters.is_empty());
        assert!(highlights.days.is_empty());
    }

    #[test]
    fn unhighlight_range_removes_day_entries() {
        let mut highlights = Highlights::default();

        highlights.highlight_range_for_day("John", 3, &[16, 17], 75);
        highlights.unhighlight_range("John", 3, &[16]);

        assert!(!highlights.is_highlighted("John", 3, 16));
        assert!(highlights.is_highlighted("John", 3, 17));
        assert!(!highlights.days[&75][&Highlights::key("John", 3)].contains(&16));
        assert!(highlights.days[&75][&Highlights::key("John", 3)].contains(&17));
    }
}
