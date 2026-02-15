//! External paths and related functionality
//!
//! The [Paths] struct is State managed by the [rocket] server, since it is needed to create
//! new [crate::client::Client]s.
use home::home_dir;
use itertools::Itertools;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

/// Default config dir for the application
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        return PathBuf::from("test");
    }

    home_dir()
        .expect("Couldn't find the home dir!")
        .join(".config/igneous-md/")
});

/// Default css dir for the application
pub static CSS_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_PATH.join("css"));

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

/// Paths used by the application
#[derive(Clone, Debug)]
pub struct Paths {
    /// The dir containing the config for the application. See [default_config_dir()]
    ///
    /// Currently this isn't very important, since we primarily care about the [Self::css_dir]
    config_dir: PathBuf,
    /// The dir containing the css files
    css_dir: PathBuf,
    /// The first css file every client receives.
    ///
    /// This is not an actual path on disk, but rather the API path for the css file
    default_css: PathBuf,
    /// The default md path sent to new clients
    default_md: PathBuf,
}

impl Paths {
    /// Attempt to create a new [Paths]
    ///
    /// This can fail, only if no `default_css` is supplied, since it needs to read css files from
    /// disk.
    pub fn new(
        default_md: PathBuf,
        css_dir: PathBuf,
        default_css: Option<PathBuf>,
    ) -> io::Result<Self> {
        Ok(Self {
            default_md,
            config_dir: CONFIG_PATH.clone(),
            default_css: default_css.unwrap_or(Self::determine_default_css(&css_dir)?),
            css_dir,
        })
    }
    /// Getter function for [Self::default_md]
    pub fn get_default_md(&self) -> PathBuf {
        self.default_md.clone()
    }

    /// Getter function for [Self::config_dir]
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    /// Getter function for [Self::css_dir]
    pub fn get_css_dir(&self) -> PathBuf {
        self.css_dir.clone()
    }

    /// Getter function for [Self::default_css]
    pub fn get_default_css(&self) -> PathBuf {
        self.default_css.clone()
    }

    /// Used by [Self::new] to set [Self::default_css] by reading `css_dir`
    ///
    /// If there are no css files it will return an empty [PathBuf]
    fn determine_default_css(css_dir: &Path) -> io::Result<PathBuf> {
        let all_css = read_css_dir(css_dir)?;

        let default_css = all_css.first().map_or(PathBuf::new(), |p| p.to_path_buf());

        log::info!(
            "Automatically set default_css to: {}",
            default_css.to_string_lossy()
        );

        Ok(default_css)
    }
}
