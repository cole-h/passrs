//! Fancy user interface
//!
//! # ui
//!
//! This module is used to display a fancy selection window when more than one
//! entry is found when using subcommands like [`show`] or [`otp-code`].
//!
//! [`show`]: ../subcmds/show/index.html
//! [`otp-code`]: ../subcmds/otp/code/index.html

use std::env;
use std::io;
use std::io::Write;

use anyhow::{Context, Result};
use termion::color;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::style;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;

use self::event::{Event, Events};
use crate::clipboard;
use crate::consts::STORE_LEN;
use crate::util;
use crate::PassrsError;

#[derive(Debug)]
pub enum UiResult {
    Success(String),
    CopiedToClipboard(String),
    SpawnEditor(String),
    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug, Default)]
struct Ui {
    entries: Vec<String>,
    selected: Option<usize>,
}

impl Ui {
    /// `entries` is a Vec containing the items to display as a part of the
    /// SelectableList
    fn new(mut entries: Vec<String>) -> Ui {
        assert!(!entries.is_empty());

        let entries: Vec<String> = entries
            .iter_mut()
            // We don't want to display the path to the password store or
            // extension, so chop those parts off
            .map(|entry| {
                if entry.ends_with(".gpg") {
                    entry.truncate(entry.len() - 4);
                }
                // Don't show PASSWORD_STORE_DIR in entry name
                entry[*STORE_LEN..].to_owned()
            })
            .collect();

        Ui {
            entries,
            selected: Some(0),
        }
    }
}

/// +-<binary name>--------------------------------------------+
/// | Found <x> matching secrets. Please select an entry.      |
/// +----------------------------------------------------------+
/// | > entry 1 <-- as selected entry                          |
/// | entry 2                                                  |
/// | entry 3                                                  |
/// +----------------------------------------------------------+
/// | <↑/↓> to change the selection, <→> to show, <←> to copy, |
/// | <e> to edit, <ESC> to quit                               |
/// +----------------------------------------------------------+
fn display_matches(matches: Vec<String>) -> Result<UiResult> {
    let bin_path = env::current_exe()?;
    let binary_name = bin_path
        .file_name()
        .with_context(|| "Option did not contain a value.")?
        .to_str()
        .with_context(|| "Option did not contain a value.")?;

    let mut app = Ui::new(matches.clone());
    let mut entry = None;
    let events = Events::new();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    loop {
        // TODO: once terminal width is smaller than max_width, reflow
        // paragraphs
        let size = terminal.size()?;

        terminal.draw(|mut frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // number of cells
                        Constraint::Percentage(50),
                        Constraint::Length(3), // number of cells
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(size);

            Paragraph::new(
                [Text::raw(format!(
                    "Found {} matching secrets. Please select an entry.",
                    app.entries.len()
                ))]
                .iter(),
            )
            .block(
                Block::default()
                    .title(binary_name)
                    .title_style(Style::default().fg(Color::Red))
                    .borders(Borders::ALL),
            )
            .wrap(true)
            .render(&mut frame, chunks[0]);
            SelectableList::default()
                .block(Block::default().borders(Borders::NONE))
                .items(&app.entries)
                .select(app.selected)
                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut frame, chunks[1]);
            Paragraph::new(
                [Text::raw(
                    "<↑/↓> to change the selection, <→> to show, <←> to copy, <e> to edit, <ESC> or <q> to quit"
                )]
                .iter(),
            )
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .render(&mut frame, chunks[2]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Esc => break,
                Key::Up => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(selected)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Left => {
                    entry = app.selected;

                    if let Some(entry) = app.selected {
                        let entry = matches[entry].to_owned();
                        let contents = util::decrypt_file_into_strings(&entry)?;

                        clipboard::clip(&contents[0], false)?;

                        return Ok(UiResult::CopiedToClipboard(entry));
                    }
                }
                Key::Down => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.entries.len() - 1 {
                            Some(selected)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Char('\n') | Key::Right => {
                    entry = app.selected;

                    break;
                }
                Key::Char('e') => {
                    entry = app.selected;

                    if let Some(entry) = app.selected {
                        let entry = matches[entry].to_owned();

                        return Ok(UiResult::SpawnEditor(entry));
                    }
                }
                Key::PageDown => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.entries.len() - 1 {
                            Some(selected)
                        } else {
                            Some(selected.saturating_add(10))
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::PageUp => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected.saturating_sub(10))
                        } else {
                            Some(selected)
                        }
                    } else {
                        Some(0)
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    terminal.show_cursor()?;
    drop(terminal);
    io::stdout().flush()?;

    // If user didn't select an entry with enter or right arrow, it was a cancellation
    if let Some(entry) = entry {
        Ok(UiResult::Success(matches[entry].to_owned()))
    } else {
        Err(PassrsError::UserAbort.into())
    }
}

pub fn display_matches_for_target(target: &str) -> Result<UiResult> {
    let matches = util::find_matches(target)?;

    if matches.len() == 1 {
        return Ok(UiResult::Success(matches[0].to_owned()));
    }

    eprintln!(
        "{yellow}Entry '{}' not found. Starting search...{reset}\n",
        &target,
        yellow = color::Fg(color::Yellow),
        reset = style::Reset
    );

    Ok(self::display_matches(matches)?)
}

mod event {
    // https://github.com/fdehau/tui-rs/blob/0168442c224bd3cd23f1d2b6494dd236b556a124/examples/util/event.rs
    // Original work Copyright (c) 2016 Florian Dehau
    // Modified work Copyright (c) 2019 Cole Helbling
    //
    // Permission is hereby granted, free of charge, to any person obtaining a copy
    // of this software and associated documentation files (the "Software"), to deal
    // in the Software without restriction, including without limitation the rights
    // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    // copies of the Software, and to permit persons to whom the Software is
    // furnished to do so, subject to the following conditions:
    //
    // The above copyright notice and this permission notice shall be included in all
    // copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    // SOFTWARE.

    use std::io;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use termion::event::Key;
    use termion::input::TermRead;

    #[derive(Debug, Clone, Copy)]
    struct Config {
        pub exit_key: Key,
        pub tick_rate: Duration,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                exit_key: Key::Char('q'),
                tick_rate: Duration::from_micros(16666),
            }
        }
    }

    pub enum Event<I> {
        Input(I),
        Tick,
    }

    /// A small event handler that wrap termion input and tick events. Each event
    /// type is handled in its own thread and returned to a common `Receiver`
    pub struct Events {
        rx: mpsc::Receiver<Event<Key>>,
    }

    impl Events {
        pub fn new() -> Events {
            Events::with_config(Config::default())
        }

        fn with_config(config: Config) -> Events {
            let (tx, rx) = mpsc::channel();
            {
                let tx = tx.clone();
                thread::spawn(move || {
                    let stdin = io::stdin();
                    for evt in stdin.keys() {
                        if let Ok(key) = evt {
                            if tx.send(Event::Input(key)).is_err() {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                    }
                });
            }
            thread::spawn(move || {
                loop {
                    // NOTE: This returns a SendError if the user ends the UI (one
                    // way or another) in between ticks.
                    let _ = tx.send(Event::Tick);
                    thread::sleep(config.tick_rate);
                }
            });

            Events { rx }
        }

        pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
            self.rx.recv()
        }
    }
}
