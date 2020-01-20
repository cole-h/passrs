use anyhow::Result;
use qrcode::QrCode;

use crate::clipboard;
use crate::consts::STORE_LEN;
use crate::ui;
use crate::ui::UiResult;
use crate::util;
use crate::Flags;
use crate::PassrsError;

use super::validate;

pub(crate) fn uri(secret_name: String, flags: Flags) -> Result<()> {
    let clip = flags.clip;
    let qr = flags.qrcode;
    let file = ui::display_matches_for_target(&secret_name)?;

    if let UiResult::Success(file) = file {
        let lines = util::decrypt_file_into_strings(&file)?;
        let file = file[*STORE_LEN..file.len() - 4].to_owned();
        let mut ret = Err(PassrsError::NoUriFound(file).into());

        for otp in lines {
            if validate::validate(&otp).is_ok() {
                ret = Ok(());

                if clip {
                    clipboard::clip(otp, false)?;
                } else if qr {
                    let code = QrCode::new(otp.as_bytes())?;
                    let rows = self::qr_grid(code);

                    println!();
                    for row in rows.iter() {
                        for pixel in row.iter() {
                            print!("{}", pixel);
                        }
                        println!();
                    }
                } else {
                    println!("{}", otp);
                }

                break;
            }
        }

        ret
    } else {
        Err(PassrsError::NoMatchesFound(secret_name).into())
    }
}

#[derive(Debug, Clone)]
struct Cell {
    ch: char,
    fg: u8,
    bg: u8,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            ch: ' ',
            fg: 30,
            bg: 47,
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\x1b[{};{}m{}\x1b[0;0m", self.fg, self.bg, self.ch)
    }
}

// https://github.com/calum/terminal_qrcode/blob/9516ac5d66c082edf396bec2bbdb6189896ac65b/src/lib.rs#L23
// Original work Copyright (c) 2019 Calum Forster
// Modified work Copyright (c) 2020 Cole Helbling
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
fn qr_grid(qr: QrCode) -> Vec<Vec<Cell>> {
    let width = qr.width();
    let pixels = qr.to_colors();
    let mut rows = vec![vec![Cell::default(); width + 2]; (width / 2) + 2];

    for (i, pixel) in pixels.iter().enumerate() {
        let x = i % width;
        let y = i / width;
        let bg = pixel.select((30, 40), (37, 47));

        let fg = match width % 2 {
            0 => bg.0,
            1 => 37,
            _ => unreachable!(),
        };

        match y % 2 {
            0 => {
                let x = x + 1;
                let y = (y / 2) + 1;
                let bg = bg.1;

                rows[y][x] = Cell { ch: '▄', fg, bg };
            }
            1 => {
                let x = x + 1;
                let y = ((y - 1) / 2) + 1;
                let fg = bg.0;

                rows[y][x].fg = fg;
            }
            _ => unreachable!(),
        }
    }

    rows
}
