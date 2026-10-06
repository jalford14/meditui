use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct Note {
    pub book: String,
    pub chapter: u16,
    pub verses: Vec<u16>,
    pub date: String,
    pub text: String,
}

pub struct Notes {
    pub entries: Vec<Note>,
    path: PathBuf,
}

impl Notes {
    pub fn load() -> io::Result<Self> {
        let dir = dirs::config_dir()
            .ok_or_else(|| io::Error::other("Could not find config directory"))?;
        Self::load_from(dir.join("bible-tui/notes.json"))
    }

    pub(crate) fn load_from(path: PathBuf) -> io::Result<Self> {
        let entries = match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).map_err(io::Error::other)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        Ok(Self { entries, path })
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(&self.entries).map_err(io::Error::other)?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, json)?;
        fs::rename(temporary, &self.path)
    }
}
