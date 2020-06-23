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
use std::io::{self, Write};

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, style};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListState, Paragraph, Text};
use tui::Terminal;

use self::event::{Event, Events};
use crate::clipboard;
use crate::consts::STORE_LEN;
use crate::util;
use crate::{PassrsError, Result};

/// The amount of entries PageUp/PageDown moves the cursor by.
const PAGE_LEN: usize = 10;

#[derive(Debug)]
#[non_exhaustive]
pub enum UiResult {
    Success(String),
    CopiedToClipboard(String),
    SpawnEditor(String),
}

#[derive(Debug, Default)]
struct Ui {
    entries: Vec<String>,
    state: ListState,
}

impl Ui {
    /// `entries` is a Vec containing the items to display as a part of the
    /// SelectableList
    fn new(entries: Vec<String>) -> Ui {
        assert!(!entries.is_empty());

        let entries: Vec<String> = entries
            .iter()
            // We don't want to display the path to the password store or
            // extension, so chop those parts off
            .map(|entry| {
                // Don't show PASSWORD_STORE_DIR or .gpg in UI
                entry[*STORE_LEN..entry.rfind(".gpg").unwrap_or_else(|| entry.len())].to_owned()
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(0));

        Ui { entries, state }
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
        .ok_or("Binary path ends in `..`")?
        .to_str()
        .ok_or("Filename was invalid unicode")?;

    let mut app = Ui::new(matches.clone());
    let mut entry = None;
    let max = app.entries.len() - 1;
    let events = Events::new();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    loop {
        // FIXME: once terminal width is smaller than max_width, reflow
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

            let heading = [Text::raw(format!(
                "Found {} matching secrets. Please select an entry.",
                max + 1
            ))];
            let entries = app.entries.iter().map(Text::raw);
            let directions = [
                Text::raw("<↑/↓> to change the selection, <→> to show, <←> to copy, <e> to edit, <ESC> or <q> to quit")
            ];

            let header = Paragraph::new(heading.iter())
                .block(
                    Block::default()
                        .title(binary_name)
                        .title_style(Style::default().fg(Color::Red))
                        .borders(Borders::ALL),
                )
                .wrap(true);
            let list = List::new(entries)
                .block(Block::default().borders(Borders::NONE))
                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            let footer = Paragraph::new(directions.iter())
                .block(Block::default().borders(Borders::ALL))
                .wrap(true);

            frame.render_widget(header, chunks[0]);
            frame.render_stateful_widget(list, chunks[1], &mut app.state);
            frame.render_widget(footer, chunks[2]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Esc => break,
                Key::Up => {
                    if let Some(selected) = app.state.selected() {
                        if selected > 0 {
                            app.state.select(Some(selected - 1));
                        } else {
                            app.state.select(Some(selected));
                        }
                    } else {
                        app.state.select(Some(0));
                    }
                }
                Key::Left => {
                    entry = app.state.selected();

                    if let Some(entry) = app.state.selected() {
                        let entry = matches[entry].to_owned();
                        let contents = util::decrypt_file_into_strings(&entry)?;

                        clipboard::clip(&contents[0], false)?;

                        return Ok(UiResult::CopiedToClipboard(entry));
                    }
                }
                Key::Down => {
                    if let Some(selected) = app.state.selected() {
                        if selected >= max {
                            app.state.select(Some(selected));
                        } else {
                            app.state.select(Some(selected + 1));
                        }
                    } else {
                        app.state.select(Some(0));
                    }
                }
                Key::Char('\n') | Key::Right => {
                    entry = app.state.selected();

                    break;
                }
                Key::Char('e') => {
                    entry = app.state.selected();

                    if let Some(entry) = app.state.selected() {
                        let entry = matches[entry].to_owned();

                        return Ok(UiResult::SpawnEditor(entry));
                    }
                }
                Key::PageDown => {
                    if let Some(selected) = app.state.selected() {
                        if selected >= max || selected.saturating_add(PAGE_LEN) >= max {
                            app.state.select(Some(max));
                        } else {
                            app.state.select(Some(selected.saturating_add(PAGE_LEN)));
                        }
                    } else {
                        app.state.select(Some(0));
                    }
                }
                Key::PageUp => {
                    if let Some(selected) = app.state.selected() {
                        if selected > 0 {
                            app.state.select(Some(selected.saturating_sub(PAGE_LEN)));
                        } else {
                            app.state.select(Some(selected));
                        }
                    } else {
                        app.state.select(Some(0));
                    }
                }
                Key::Home => {
                    app.state.select(Some(0));
                }
                Key::End => {
                    app.state.select(Some(max));
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

    if termion::is_tty(&io::stdout()) {
        writeln!(
            io::stderr(),
            "{yellow}Entry '{}' not found. Starting search...{reset}\n",
            &target,
            yellow = color::Fg(color::Yellow),
            reset = style::Reset
        )?;
    }

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
