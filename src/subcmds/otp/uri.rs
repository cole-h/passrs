use std::fs::File;
use std::io::Write;

use anyhow::{Context, Result};
use qrcode::render::svg;
use qrcode::QrCode;

use crate::clipboard;
use crate::ui::{self, UiResult};
use crate::util;

// TODO ?
pub fn uri(clip: bool, qrcode: Option<String>, pass_name: String) -> Result<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    if let UiResult::Success(file) = file {
        let otp = util::decrypt_file_into_strings(file)?;
        let otp = otp.first().with_context(|| "Vec was empty")?.trim();

        if clip {
            clipboard::clip(otp)?;
        }
        if let Some(qrcode) = qrcode {
            let code = QrCode::new(otp.as_bytes())?;
            let svg_xml = code.render::<svg::Color>().build();

            let mut file = File::create(&qrcode)?;
            file.write_all(svg_xml.as_bytes())?;

            println!("Image created at {}.", qrcode);
        }
    }

    Ok(())
}
