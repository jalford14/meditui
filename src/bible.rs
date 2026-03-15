use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct BookJson {
    #[allow(dead_code)]
    book: String,
    chapters: Vec<ChapterJson>,
}

#[derive(Deserialize)]
struct ChapterJson {
    chapter: String,
    verses: Vec<VerseJson>,
}

#[derive(Deserialize)]
struct VerseJson {
    verse: String,
    text: String,
}

#[derive(Clone)]
pub struct Verse {
    pub number: u16,
    pub text: String,
}

pub struct BookData {
    pub chapters: HashMap<u16, Vec<Verse>>,
}

pub struct Bible {
    pub books: HashMap<String, BookData>,
}

/// Convert a book name like "1 Samuel" or "Song of Solomon" to the filename stem "1Samuel" / "SongofSolomon".
fn book_to_filename(book: &str) -> String {
    book.replace(' ', "")
}

impl Bible {
    /// Load only the chapters needed for today's reading from individual book JSON files.
    pub fn load_chapters(
        data_dir: &Path,
        translation: &str,
        chapters: &[(String, u16)],
    ) -> Bible {
        let translation_dir = data_dir.join(translation);
        let mut books: HashMap<String, BookData> = HashMap::new();

        // Deduplicate which books we need to load
        let mut needed_books: HashMap<String, Vec<u16>> = HashMap::new();
        for (book, ch) in chapters {
            needed_books
                .entry(book.clone())
                .or_default()
                .push(*ch);
        }

        for (book_name, needed_chapters) in &needed_books {
            let filename = format!("{}.json", book_to_filename(book_name));
            let path = translation_dir.join(&filename);

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue, // skip missing books gracefully
            };

            let book_json: BookJson = match serde_json::from_str(&content) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let mut chapter_map = HashMap::new();
            for ch in book_json.chapters {
                let ch_num: u16 = match ch.chapter.parse() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                if needed_chapters.contains(&ch_num) {
                    let verses: Vec<Verse> = ch
                        .verses
                        .into_iter()
                        .filter_map(|v| {
                            Some(Verse {
                                number: v.verse.parse().ok()?,
                                text: v.text,
                            })
                        })
                        .collect();
                    chapter_map.insert(ch_num, verses);
                }
            }

            books.insert(book_name.clone(), BookData { chapters: chapter_map });
        }

        Bible { books }
    }

    pub fn get_chapter(&self, book: &str, chapter: u16) -> Option<&Vec<Verse>> {
        self.books.get(book).and_then(|b| b.chapters.get(&chapter))
    }
}
