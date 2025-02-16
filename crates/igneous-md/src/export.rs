//! Just contains [export()].
use std::{io, path::PathBuf};

use crate::paths::default_config_dir;
/// Saves the given html string to disk
///
/// The default location is in the users config dir, with the name:
/// `html-export-<year>-<month>-<day>-<hour>-<minute>-<second>.html`
///
/// It is possible that one file overwrites another if the user happens to press the export button
/// twice in one second, but this should never happen in normal use.
///
/// Alternatively, if `other_path` is given, it will be used as the path to save the file to.
pub fn export(html: String, other_path: Option<PathBuf>) -> io::Result<()> {
    let path = match other_path {
        Some(path) if path.is_dir() => path.join(format!(
            "html-export-{}.html",
            chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
        )),
        Some(path) => path,
        None => default_config_dir().join(format!(
            "html-export-{}.html",
            chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
        )),
    };

    std::fs::write(&path, html)?;

    log::info!("Exported HTML to: {}", path.to_string_lossy());

    Ok(())
}
