use std::fs;
use std::path::Path;

fn main() {
    let kjv_dir = Path::new("data/kjv");
    let merged_path = Path::new("data/bible_merged.json");

    if !kjv_dir.exists() {
        panic!("data/kjv/ not found. Run ./setup.sh first to download Bible data.");
    }

    // Collect all JSON files
    let mut entries: Vec<_> = fs::read_dir(kjv_dir)
        .expect("Failed to read data/kjv/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();

    if entries.is_empty() {
        panic!("No JSON files found in data/kjv/. Run ./setup.sh first.");
    }

    // Sort by filename for deterministic output
    entries.sort_by_key(|e| e.file_name());

    // Read each file and parse as a JSON value, then collect into an array
    let mut books: Vec<serde_json::Value> = Vec::new();
    for entry in &entries {
        let path = entry.path();
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
        let value: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        books.push(value);

        // Tell cargo to rerun if this file changes
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let merged = serde_json::to_string(&books).expect("Failed to serialize merged bible");
    fs::write(merged_path, merged).expect("Failed to write data/bible_merged.json");

    println!("cargo:rerun-if-changed=data/kjv");
}
