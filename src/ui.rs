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
use zeroize::Zeroize;

use crate::clipboard;
use crate::consts::PASSWORD_STORE_DIR;
use crate::event::{Event, Events};
use crate::util;
use crate::PassrsError;

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
    fn new(mut entries: Vec<String>) -> Self {
        assert!(!entries.is_empty());

        let entries = entries
            .iter_mut()
            // We don't want to display the path to the password store or
            // extension, so chop those parts off
            .map(|entry| {
                if entry.ends_with(".gpg") {
                    entry.truncate(entry.len() - 4);
                }
                // Don't show PASSWORD_STORE_DIR in entry name
                entry[PASSWORD_STORE_DIR.len()..].to_owned()
            })
            .collect::<Vec<_>>();

        Ui {
            entries,
            selected: Some(0),
        }
    }

    // TODO: use this to resize the UI based on length
    // fn max_len(&self) -> Option<u16> {
    //     let mut max_len = 0;
    //     for entry in &self.entries {
    //         max_len = if entry.len() > max_len {
    //             entry.len()
    //         } else {
    //             max_len
    //         };
    //     }

    //     if max_len == 0 {
    //         None
    //     } else {
    //         Some(max_len as u16)
    //     }
    // }
}

/// +-<binary name>--------------------------------------------+
/// | Found <x> matching secrets. Please select an entry.      |
/// +----------------------------------------------------------+
/// | > entry 1 <-- as selected entry                          |
/// | entry 2                                                  |
/// | entry 3                                                  |
/// +----------------------------------------------------------+
/// | <↑/↓> to change the selection, <→> to show, <←> to copy, |
/// | ~~<s> to sync~~, <e> to edit, <ESC> to quit              |
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

    // `terminal` gets dropped at the end of the scope, allowing stdout to work
    // as expected
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();
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
                            // Constraint::Length(par1), // number of cells
                            Constraint::Percentage(50),
                            Constraint::Length(3), // number of cells
                            // Constraint::Length(par2), // number of cells
                            Constraint::Percentage(50),
                        ]
                            .as_ref(),
                    )
                    .split(size);
                Paragraph::new(
                    vec![Text::raw(format!(
                        "Found {} matching secrets. Please select an entry.",
                        app.entries.len()
                    ))]
                        .iter(),
                )
                    .block(Block::default()
                           .title(binary_name)
                           .title_style(Style::default().fg(Color::Red))
                           .borders(Borders::ALL))
                    // TODO: change constraints when the terminal shrinks/grows
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
                    vec![Text::raw(
                        "<↑/↓> to change the selection, <→> to show, <←> to copy, <e> to edit, <ESC> or <q> to quit",
                    )]
                        .iter(),
                )
                    .block(Block::default().borders(Borders::ALL))
                    // TODO: change constraints when the terminal shrinks/grows
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
                    // dispatch to decrypt file and copy to clipboard -- ONLY
                    // FIRST LINE (some people use one entry for passwords,
                    // notes, etc., with password as the first line)
                    entry = app.selected;
                    if let Some(entry) = app.selected {
                        let entry = matches[entry].to_owned();
                        let mut contents = util::decrypt_file_into_strings(&entry)?;
                        clipboard::clip(&contents[0])?;
                        contents.zeroize();

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

    // drop terminal so we can use stdout as usual
    drop(terminal);
    io::stdout().flush()?;

    // If user didn't select an entry with enter or right arrow, it was a cancellation
    if let Some(entry) = entry {
        // println!("{}", &matches[entry]);
        Ok(UiResult::Success(matches[entry].to_owned()))
    } else {
        // println!(
        //     "{red}Error: user aborted{reset}",
        //     red = color::Fg(color::Red),
        //     reset = style::Reset
        // );
        Err(PassrsError::UserAbort.into())
    }
}

pub fn display_matches_for_target(target: &str) -> Result<UiResult> {
    let matches = util::find_target_single(target)?;

    if matches.len() == 1 {
        return Ok(UiResult::Success(matches[0].to_owned()));
    }

    // TODO: color or no color?
    // if I can find a way to add color to the failure display messages, color;
    // else, no color
    eprintln!(
        "{yellow}Entry '{}' not found. Starting search...{reset}\n",
        &target,
        yellow = color::Fg(color::Yellow),
        reset = style::Reset
    );

    Ok(display_matches(matches)?)
}

// pub fn display_matches_for_targets(matches: Vec<String>) -> Result<UiResult> {
//     if matches.len() == 1 {
//         return Ok(UiResult::Success(matches[0].to_owned()));
//     }

//     Ok(display_matches(matches)?)
// }
