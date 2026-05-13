//! Shared TUI infrastructure.
//!
//! Owns the enter/leave dance for raw mode + alternate screen + mouse
//! capture so that the per-command TUIs (`tree`, `todo`) do not duplicate
//! it. A panic hook is registered in `main` so a crashed TUI never leaves
//! the terminal in raw mode.
//!
//! The full TUI commands themselves are stubbed; this file ports the
//! plumbing so they can be written without re-doing terminal setup.

#![allow(dead_code)]

pub mod filter;
pub mod tags;
pub mod text;
pub mod theme;

use std::io::{Stdout, stdout};

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
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
