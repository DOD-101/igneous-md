//! External paths and related functionality
use dirs::config_dir;
use itertools::Itertools;
use lightningcss::{
    bundler::{Bundler, FileProvider},
    printer::PrinterOptions,
    stylesheet::ParserOptions,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::config::CssEntry;

pub const SERVER_PORT_FILE: &str = "/tmp/igneous-md";

/// Default config dir for the application
///
/// <div class="warning">
/// Do not use this to access the config, this is the default value, which may have been
/// overridden by the user.
/// </div>
pub static DEFAULT_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        return std::env::current_dir()
            .expect("Failed to get cwd")
            .join("test");
    }

    config_dir()
        .expect("Couldn't find the home dir!")
        .join("igneous-md/")
});

/// Bundle and minify a CSS file using lightningcss
///
/// This processes @import rules and inlines them, then minifies the result.
fn bundle_and_minify(css_path: &Path) -> Result<String, String> {
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let mut stylesheet = bundler
        .bundle(css_path)
        .map_err(|e| format!("Failed to bundle CSS: {e}"))?;

    stylesheet
        .minify(lightningcss::stylesheet::MinifyOptions::default())
        .map_err(|e| format!("Failed to minify CSS: {e}"))?;

    let result = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        })
        .map_err(|e| format!("Failed to print CSS: {e}"))?;

    Ok(result.code)
}

/// Will attempt to read the given `css_dir` and organize the output
///
/// This function will:
///
/// 1. Only include `.css` files from the top-level directory (not subdirectories like hljs/)
///
/// 2. Bundle each file with its @import dependencies
///
/// 3. Minify the bundled result
///
/// 4. Return entries with paths and bundled+minified file contents
///
/// 5. Sort them by their name
pub fn read_css_dir(css_dir: &Path) -> io::Result<Vec<CssEntry>> {
    let entries: Vec<CssEntry> = fs::read_dir(css_dir)?
        .filter_map(|possible_entry| {
            let path = possible_entry.ok()?.path();

            if path.is_file() && path.extension().is_some_and(|s| s == "css") {
                let content = match bundle_and_minify(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!("Failed to bundle/minify CSS file {}: {}", path.display(), e);
                        return None;
                    }
                };

                return Some(CssEntry { path, content });
            }

            None
        })
        .sorted_by_key(|entry| {
            PathBuf::from(
                entry
                    .path
                    .file_name()
                    .expect("We checked that all entries are files."),
            )
        })
        .collect();

    Ok(entries)
}

/// Attempt to delete the tmp port file
///
/// If this fails it will log a warning.
///
/// See:
/// - [SERVER_PORT_FILE]
pub fn attempt_delete_port_file() {
    if let Err(e) = fs::remove_file(SERVER_PORT_FILE) {
        log::warn!("Failed to remove tmp port file: {e}")
    }
}

/// Attempt to write the tmp port file
///
/// If this fails it will log a warning
///
/// See:
/// - [SERVER_PORT_FILE]
pub fn attempt_write_port_file(port: u16) {
    if let Err(e) = fs::write(SERVER_PORT_FILE, port.to_string()) {
        log::warn!("Failed to write tmp port file: {e}")
    }
}
