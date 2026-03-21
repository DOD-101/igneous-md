//! External paths and related functionality
//!
//! The [Paths] struct is State managed by the [rocket] server, since it is needed to create
//! new [crate::client::Client]s.
use dirs::config_dir;
use itertools::Itertools;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

/// Default config dir for the application
///
/// > [WARNING]
/// > Do not use this to access the config, this is the default value, which may have been
/// > overridden by the user. See [`PATHS`] for that.
pub static DEFAULT_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        return PathBuf::from("test");
    }

    config_dir()
        .expect("Couldn't find the home dir!")
        .join("igneous-md/")
});

/// Will attempt to read the given `css_dir` and organize the output
///
/// This function will:
///
/// 1. Only include `.css` files
///
/// 2. Return only names prefixed with `/css`
///
/// 3. Sort them by their name
pub fn read_css_dir(css_dir: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(css_dir)?
        .filter_map(|possible_entry| {
            let path = possible_entry.ok()?.path();

            if path.is_file() && path.extension().is_some_and(|s| s == "css") {
                return Some(
                    PathBuf::from("/css").join(
                        path.strip_prefix(css_dir)
                            .expect("We read the files from the css_dir."),
                    ),
                );
            }

            None
        })
        .sorted_by_key(|p| {
            PathBuf::from(
                p.file_name()
                    .expect("We checked that all entries are files."),
            )
        })
        .collect())
}
