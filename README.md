# meditui

A terminal Bible reader following Robert Murray M'Cheyne's daily reading plan. Built with [Ratatui](https://ratatui.rs), runs fully offline with the KJV.

```
┌─────────────────────────────────────────────────────────┐
│  McHeyne Day 54 — February 23   Family | Secret         │
├─────────────────────────────────────────────────────────┤
│  Exodus 6 | Luke 9 | Job 23 | 1 Corinthians 10          │
├─────────────────────────────────────────────────────────┤
│                      Exodus 6                           │
│                                                         │
│  > 1  Then the LORD said unto Moses, Now shalt thou     │
│       see what I will do to Pharaoh...                  │
│    2  And God spake unto Moses, and said unto him,      │
│       I am the LORD:                                    │
│    3  And I appeared unto Abraham, unto Isaac, and      │
│       unto Jacob, by the name of God Almighty...        │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  NORMAL | Exodus 6 (30 verses)                          │
│  j/k:move  h/l:chapter  v:select  Enter:highlight       │
└─────────────────────────────────────────────────────────┘
```

## About the plan

M'Cheyne (1813-1843) was a Scottish minister who designed a calendar for reading through the entire Bible in one year. Each day has four chapters split across two tracks:

- **Family** — two readings suited for group devotion (OT historical books + Gospels/Epistles)
- **Secret** — two readings for private study (prophets/wisdom literature + Acts-Revelation/Psalms)

The result: the Old Testament once and the New Testament + Psalms twice per year.

## Setup

Requires Rust and an internet connection for the initial data download.

```sh
git clone https://github.com/yourusername/meditui.git
cd meditui
./setup.sh      # downloads KJV text (~1.9MB) — run once
cargo run        # or: cargo run --release
```

The KJV text (public domain) is sourced from [aruljohn/Bible-kjv](https://github.com/aruljohn/Bible-kjv) and embedded into the binary at compile time. No network access needed after build.

## Keybindings

### Normal mode

| Key | Action |
|-----|--------|
| `j` / `Down` | Next verse |
| `k` / `Up` | Previous verse |
| `H` | Jump to first visible verse |
| `L` | Jump to last visible verse |
| `gg` | Jump to first verse |
| `G` | Jump to last verse |
| `Ctrl-d` | Half page down |
| `Ctrl-u` | Half page up |
| `l` / `Tab` | Next chapter |
| `h` / `Shift-Tab` | Previous chapter |
| `a` | Open highlight archive |
| `r` | Return to today's readings |
| `v` | Enter visual selection mode |
| `Enter` | Toggle highlight on current verse |
| `q` | Quit |

### Highlight archive

| Key | Action |
|-----|--------|
| `j` / `Down` | Next highlighted reference |
| `k` / `Up` | Previous highlighted reference |
| `Enter` | Open selected highlighted reference |
| `Esc` | Return to reading |
| `q` | Quit |

### Visual mode

| Key | Action |
|-----|--------|
| `j` / `k` | Extend selection |
| `y` | Highlight selected verses (yellow) |
| `d` | Remove highlight from selection |
| `Esc` | Cancel selection |

## Highlights

Verse highlights persist across sessions at `~/.config/bible-tui/highlights.json`. Use `Enter` to toggle a single verse, or `v` to select a range then `y` to highlight / `d` to clear. Press `a` to browse highlights grouped by M'Cheyne reading day; older highlight files are grouped by the reading days that include each highlighted chapter.

## License

MIT
