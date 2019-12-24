use failure::err_msg;
use failure::Fallible;

use termion::color::Fg;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;

// use crate::error::*;
// use crate::Result;
use crate::consts::PASSWORD_STORE_DIR;
use crate::event::{Event, Events};
use crate::utils;

#[derive(Debug, Default)]
struct Ui {
    entries: Vec<String>,
    selected: Option<usize>,
}

impl Ui {
    fn new(entries: Vec<String>) -> Self {
        assert!(entries.len() > 0);

        Ui {
            entries: entries,
            selected: Some(0),
        }
    }

    fn max_len(&self) -> Option<usize> {
        let mut max_len = 0;
        for entry in &self.entries {
            max_len = if entry.len() > max_len {
                entry.len()
            } else {
                max_len
            };
        }

        if max_len == 0 {
            None
        } else {
            Some(max_len)
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
/// | ~~<s> to sync~~, <e> to edit, <ESC> to quit              |
/// +----------------------------------------------------------+
pub fn display_matches<S>(target: S) -> Fallible<()>
where
    S: Into<String>,
{
    let target = target.into();

    let bin_path = std::env::current_exe()?;
    let binary_name = bin_path
        .file_name()
        .ok_or(err_msg("Option did not contain a value."))?
        .to_str()
        .ok_or(err_msg("Option did not contain a value."))?;
    let matches = utils::search_entries(&target)?
        .iter()
        // we don't want to display the path to the password store, so chop that off
        .map(|entry| entry[PASSWORD_STORE_DIR.len()..].to_string())
        .collect::<Vec<_>>();
    let mut app = Ui::new(matches);
    let mut entry = None;

    println!(
        "{}Entry '{}' not found. Starting search...{}",
        Fg(termion::color::Yellow),
        &target,
        Fg(termion::color::Reset)
    );

    // `terminal` gets dropped at the end of the scope, allowing stdout to work
    // as expected
    {
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let events = Events::new();
        terminal.hide_cursor()?;

        loop {
            // TODO: max width should be app.max_len()
            let _ = app.max_len();
            let size = terminal.size()?;

            terminal.draw(|mut frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // number of cells
                            Constraint::Percentage(30),
                            Constraint::Length(3), // number of cells
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
                    // .wrap(true)
                    .render(&mut frame, chunks[0]);
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Matching entries"))
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
                    // .wrap(true)
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
                        // TODO: dispatch to decrypt file and copy to clipboard
                        // -- ONLY FIRST LINE (some people use one entry for
                        // passwords, notes, etc., with password as the first
                        // line)
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
                        // TODO: spawn editor
                        break;
                    }
                    _ => {}
                },
                Event::Tick => {}
            }
        }
        terminal.show_cursor()?;
    }

    // If user didn't select entry with enter or right arrow
    if let Some(entry) = entry {
        // TODO: decrypt the file, copy password to clipboard
        println!("{}", app.entries[entry]);
    } else {
        println!(
            "{}Error: user aborted{}",
            Fg(termion::color::Red),
            Fg(termion::color::Reset)
        );
    }

    Ok(())
}
