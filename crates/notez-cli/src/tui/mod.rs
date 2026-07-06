//! Shared TUI infrastructure.
//!
//! Owns the enter/leave dance for raw mode + alternate screen + mouse
//! capture so that the per-command TUIs (`tree`, `todo`) do not duplicate
//! it. A panic hook is registered in `main` so a crashed TUI never leaves
//! the terminal in raw mode.

#![allow(dead_code)]

pub mod tags;
pub mod text;
pub mod theme;
pub mod todo;
pub mod tree;

use std::io::{Stdout, stdout};

use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn enter() -> Result<TuiTerminal> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(out))?)
}

pub fn leave() -> Result<()> {
    let mut out = stdout();
    execute!(out, DisableMouseCapture, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Suspend the TUI, open `path` in the given editor command, and block
/// until it exits. The caller re-enters with [`enter`] afterwards.
pub fn open_in_editor(editor: &str, path: &std::path::Path) -> Result<()> {
    leave()?;
    std::process::Command::new(editor)
        .arg(path)
        .status()
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("failed to launch editor {editor}: {e}"))
}

/// Vim-style `:` command line, carried over from notez-cli so `:q` / `:wq`
/// muscle memory keeps working inside the TUIs.
pub struct VimCommandMode {
    pub active: bool,
    pub buffer: String,
}

impl VimCommandMode {
    pub fn new() -> Self {
        Self {
            active: false,
            buffer: String::new(),
        }
    }

    /// Process a key event. Returns `Some(command)` when Enter completes one.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        if !self.active {
            if key.code == KeyCode::Char(':') {
                self.active = true;
                self.buffer.clear();
                self.buffer.push(':');
            }
            return None;
        }
        match key.code {
            KeyCode::Enter => {
                let cmd = self.buffer.clone();
                self.active = false;
                self.buffer.clear();
                Some(cmd)
            }
            KeyCode::Esc => {
                self.active = false;
                self.buffer.clear();
                None
            }
            KeyCode::Backspace => {
                self.buffer.pop();
                if self.buffer.is_empty() {
                    self.active = false;
                }
                None
            }
            KeyCode::Char(c) => {
                self.buffer.push(c);
                None
            }
            _ => None,
        }
    }

    /// True when a completed command means "quit".
    pub fn is_quit(cmd: &str) -> bool {
        matches!(cmd, ":wq" | ":qa" | ":q" | ":q!")
    }
}
