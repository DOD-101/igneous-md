//! Just contains [export()].
use std::{ffi::OsString, io, path::PathBuf};

/// Saves the given html string to disk
///
/// The default file name is: `html-export-<year>-<month>-<day>-<hour>-<minute>-<second>.html`
///
/// It is possible that one file overwrites another if the user happens to press the export button
/// twice in one second, but this should never happen in normal use.
///
/// Alternatively, if `file_name` is given, it will be used as the name of the file to save to
/// disk.
pub fn export(html: String, dir: PathBuf, file_name: Option<OsString>) -> io::Result<()> {
    let path = match file_name {
        Some(n) => dir.join(n),
        None => dir.join(format!(
            "html-export-{}.html",
            chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
        )),
    };

    std::fs::write(&path, html)?;

    log::info!("Exported HTML to: {}", path.to_string_lossy());

    Ok(())
}
