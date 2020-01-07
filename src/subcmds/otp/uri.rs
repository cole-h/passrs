use std::fs::File;
use std::io::Write;

use anyhow::{Context, Result};
use qrcode::render::svg;
use qrcode::QrCode;

use crate::clipboard;
use crate::ui;
use crate::ui::UiResult;
use crate::util;

pub fn uri(clip: bool, qr_path: Option<String>, pass_name: String) -> Result<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    if let UiResult::Success(file) = file {
        let otp = util::decrypt_file_into_strings(file)?;
        let otp = otp.first().with_context(|| "File was empty")?.trim();

        if clip {
            clipboard::clip(otp)?;
        }
        if let Some(qr_path) = qr_path {
            let code = QrCode::new(otp.as_bytes())?;
            let svg_xml = code.render::<svg::Color>().build();

            // TODO: util::create_recursive_dirs(&qr_path)?;
            let mut file = File::create(&qr_path)?;
            file.write_all(svg_xml.as_bytes())?;

            println!("Image created at {}.", qr_path);
        }
    }

    Ok(())
}
