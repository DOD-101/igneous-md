//! Just contains [export()].
use std::{io, path::PathBuf};

use crate::paths::default_config_dir;
/// Saves the given html string to disk
///
/// The file is stored in the users config dir, with the name:
/// `html-export-<year>-<month>-<day>-<hour>-<minute>-<second>.html`
///
/// It is possible that one file overwrites another if the user happens to press the export button
/// twice in one second, but this should never happen in normal use.
pub fn export(html: String, other_path: Option<PathBuf>) -> io::Result<()> {
    // Save the HTML string to a file
    let file_name = format!(
        "html-export-{}.html",
        chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
    );

    let file_path = other_path
        .unwrap_or(PathBuf::from(default_config_dir()))
        .join(file_name);

    std::fs::write(&file_path, html)?;

    log::info!("Exported HTML to: {}", file_path.to_string_lossy());

    Ok(())
}
