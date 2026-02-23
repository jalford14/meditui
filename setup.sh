#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data"
KJV_DIR="$DATA_DIR/kjv"

mkdir -p "$KJV_DIR"

# These are the exact filenames in the aruljohn/Bible-kjv repo (no spaces)
BOOKS=(
  Genesis Exodus Leviticus Numbers Deuteronomy
  Joshua Judges Ruth
  1Samuel 2Samuel 1Kings 2Kings
  1Chronicles 2Chronicles
  Ezra Nehemiah Esther
  Job Psalms Proverbs Ecclesiastes SongofSolomon
  Isaiah Jeremiah Lamentations Ezekiel Daniel
  Hosea Joel Amos Obadiah Jonah Micah Nahum Habakkuk Zephaniah Haggai Zechariah Malachi
  Matthew Mark Luke John Acts
  Romans 1Corinthians 2Corinthians
  Galatians Ephesians Philippians Colossians
  1Thessalonians 2Thessalonians
  1Timothy 2Timothy Titus Philemon
  Hebrews James 1Peter 2Peter
  1John 2John 3John Jude Revelation
)

BASE_URL="https://raw.githubusercontent.com/aruljohn/Bible-kjv/master"

echo "Downloading KJV books..."
for book in "${BOOKS[@]}"; do
  filename="${book}.json"
  outpath="$KJV_DIR/$filename"
  if [ -f "$outpath" ]; then
    echo "  [skip] $filename (already exists)"
    continue
  fi
  echo "  [download] $filename"
  curl -sS -f -o "$outpath" "$BASE_URL/${filename}"
done

echo "Downloading McHeyne reading plan..."
MCCHEYNE_URL="https://raw.githubusercontent.com/BibleReadingPlans/bible-reading-plan-schema/master/mccheyne.json"
MCCHEYNE_PATH="$DATA_DIR/mccheyne.json"
if [ -f "$MCCHEYNE_PATH" ]; then
  echo "  [skip] mccheyne.json (already exists)"
else
  curl -sS -f -o "$MCCHEYNE_PATH" "$MCCHEYNE_URL"
  echo "  [download] mccheyne.json"
fi

echo "Done! Downloaded $(ls "$KJV_DIR"/*.json 2>/dev/null | wc -l | tr -d ' ') KJV books."
