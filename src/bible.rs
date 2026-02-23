use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct BookJson {
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

impl Bible {
    pub fn from_json(json_str: &str) -> Bible {
        let raw: Vec<BookJson> = serde_json::from_str(json_str).expect("Failed to parse bible JSON");
        let mut books = HashMap::new();

        for book_json in raw {
            let mut chapters = HashMap::new();
            for ch in book_json.chapters {
                let ch_num: u16 = ch.chapter.parse().expect("Invalid chapter number");
                let verses: Vec<Verse> = ch
                    .verses
                    .into_iter()
                    .map(|v| Verse {
                        number: v.verse.parse().expect("Invalid verse number"),
                        text: v.text,
                    })
                    .collect();
                chapters.insert(ch_num, verses);
            }
            books.insert(book_json.book, BookData { chapters });
        }

        Bible { books }
    }

    pub fn get_chapter(&self, book: &str, chapter: u16) -> Option<&Vec<Verse>> {
        self.books.get(book).and_then(|b| b.chapters.get(&chapter))
    }
}
