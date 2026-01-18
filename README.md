# gwatch

Real-time Git-powered directory monitor with line-by-line diff visualization.

[![CI](https://github.com/csaltos/gwatch/actions/workflows/ci.yml/badge.svg)](https://github.com/csaltos/gwatch/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Overview

**gwatch** is a high-performance CLI tool that provides a real-time, interactive TUI to monitor and visualize line-by-line changes across a Git repository. Unlike traditional file monitors that only report *what changed*, gwatch shows *exactly which lines changed* the moment they are saved to disk.

## Features

- **Real-time monitoring**: <50ms latency from disk write to UI visualization
- **VS Code-style diffs**: Green for additions, red for deletions
- **Interactive TUI**: Pause, scroll through history, open files in your editor
- **Theme support**: Nord, Catppuccin, Dracula, and Monochrome themes
- **Fully configurable**: JSON config at `~/.config/gwatch/config.json`
- **Git-native**: Uses libgit2 for efficient diff computation against HEAD

## Installation

```bash
# From source
cargo install --path .

# Or build locally
cargo build --release
./target/release/gwatch
```

## Usage

Navigate to any Git repository and run:

```bash
gwatch
```

### Command Line Options

```bash
gwatch [OPTIONS]

Options:
  -p, --path <PATH>  Directory to watch [default: current directory]
  -v, --verbose...   Increase log verbosity (-v, -vv, -vvv)
  -h, --help         Print help information
  -V, --version      Print version information
```

### Examples

```bash
# Watch current directory
gwatch

# Watch a specific repository
gwatch --path ~/projects/myrepo

# Watch with debug logging
gwatch -v

# Watch with trace logging
gwatch -vvv
```

### Keybindings

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume live stream |
| `↑`/`↓` or `j`/`k` | Scroll through history |
| `Enter` | Open current file in `$EDITOR` |
| `m` | Cycle diff mode (All/Unstaged/Staged) |
| `]` / `[` | Jump to next/previous hunk |
| `z` | Toggle current hunk collapsed |
| `Z` | Toggle hide all context lines |
| `r` | Toggle reviewed status for current file |
| `R` | Clear all reviewed markers |
| `d` | Open diff in external viewer |
| `t` | Open theme selector |
| `s` | Open settings editor |
| `c` | Clear event history |
| `?` | Show help panel |
| `q` or `Esc` | Quit |

### Settings Editor

Press `s` to open the in-TUI settings editor:
- Edit JSON config directly in the terminal
- `Ctrl+S` to save and apply changes
- `Esc` to cancel without saving
- Changes are applied immediately and saved to disk

### Diff Modes

gwatch supports three diff modes, cycled with `m`:

| Mode | Description |
|------|-------------|
| **All Changes** | Working tree vs HEAD (default) |
| **Unstaged** | Working tree vs Index (what's modified but not staged) |
| **Staged** | Index vs HEAD (what will be committed) |

### Hunk Navigation

For diffs with multiple hunks, gwatch provides focused navigation:

| Key | Action |
|-----|--------|
| `]` | Jump to next hunk |
| `[` | Jump to previous hunk |
| `z` | Collapse/expand current hunk |
| `Z` | Hide/show all context lines |

**Visual indicators:**
- `▼` = hunk expanded (showing all lines)
- `▶` = hunk collapsed (showing summary only)
- Focused hunk header is highlighted
- Footer shows "Hunk X/Y" position

This helps maintain mental state when reviewing large diffs with many changes.

### Review Tracking

gwatch helps you track which files you've already reviewed:

- Press `r` to mark/unmark the current file as reviewed
- Reviewed files show a "✓ Reviewed" badge in the header
- Review state persists across sessions (stored in `~/.config/gwatch/review_state.json`)
- Press `R` (Shift+r) to clear all reviewed markers and start fresh

This creates a "review loop" workflow where you can:
1. Watch for changes as they happen
2. Mark files as reviewed once you've looked at them
3. Focus on new/unreviewed changes
4. Clear reviews when starting a new session

### External Diff Viewers

gwatch can integrate with external diff viewers for enhanced visualization:

| Viewer | Command | Description |
|--------|---------|-------------|
| **delta** | `delta` | Syntax highlighting, side-by-side view |
| **difftastic** | `difft` | Structural diff, language-aware |
| **Internal** | (built-in) | TUI-based side-by-side view |

**Auto-detection:** When set to "auto", gwatch checks for available viewers in order:
1. `delta` (if installed)
2. `difftastic` (if installed)
3. Internal viewer (always available)

Press `d` to open the current file's diff in the configured viewer.

## Configuration

gwatch creates a config file at `~/.config/gwatch/config.json`:

```json
{
  "theme": {
    "name": "nord"
  },
  "editor": {
    "command": "vim",
    "args": ["+{line}", "{file}"]
  },
  "watcher": {
    "debounce_ms": 50,
    "max_events_buffer": 300,
    "ignore_patterns": ["node_modules", "dist", "build", "*.log", "target"]
  },
  "display": {
    "context_lines": 3,
    "show_line_numbers": true,
    "use_nerd_font_icons": true
  },
  "diff_viewer": {
    "viewer": "auto",
    "pager": null,
    "delta_args": ["--side-by-side"],
    "difftastic_args": []
  }
}
```

## Available Themes

- **Nord** (default) - Cool, blue-focused dark theme
- **Catppuccin Mocha** - Pastel dark theme
- **Catppuccin Frappé** - Slightly lighter pastel theme
- **Dracula** - Classic dark theme (purple-free variant)
- **Monochrome** - Minimal terminal colors

## Large File Handling

gwatch handles large files gracefully:
- Files >1MB: Diff computed with warning
- Files >10MB: Skipped entirely
- Diffs >5000 lines: Truncated to first/last 100 lines

## Requirements

- Rust 1.70+ (for building)
- Git repository (gwatch only works inside Git repos)
- macOS or Linux

## License

MIT
