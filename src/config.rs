use itertools::Itertools;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(feature = "generate_config")]
use std::io::Write;

use crate::paths::Paths;

#[derive(Debug, Clone)]
pub struct Config {
    config_dir: PathBuf,
    css_dir: PathBuf,
    css_paths: Vec<PathBuf>,
    current_css_index: usize,
}

impl Config {
    pub fn new(paths: Paths) -> io::Result<Self> {
        let mut config = Self {
            config_dir: paths.get_config_dir(),
            css_dir: paths.get_css_dir(),
            css_paths: vec![],
            current_css_index: 0,
        };

        config.update_css_paths()?;

        Ok(config)
    }

    pub fn next_css(&mut self) -> Option<PathBuf> {
        self.current_css_index = (self.current_css_index + 1) % self.css_paths.len();

        self.css_paths.get(self.current_css_index).cloned()
    }

    pub fn previous_css(&mut self) -> Option<PathBuf> {
        self.current_css_index = (self.current_css_index - 1) % self.css_paths.len();

        self.css_paths.get(self.current_css_index).cloned()
    }

    #[allow(dead_code)]
    pub fn current_css(&self) -> Option<PathBuf> {
        self.css_paths.get(self.current_css_index).cloned()
    }

    #[allow(dead_code)]
    pub fn get_css_dir(&self) -> &PathBuf {
        &self.css_dir
    }

    #[allow(dead_code)]
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Reads the css dir, updating self.css_paths
    pub fn update_css_paths(&mut self) -> io::Result<()> {
        let all_css: Vec<PathBuf> = read_css_dir(&self.css_dir)?;

        self.css_paths = all_css;

        log::info!("Updated css_paths: {:?}", self.css_paths);

        Ok(())
    }
}

/// Will attempt to read the given `css_dir` returning only css files and sorting them
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

/// Creates the css files on disk
///
/// Used if there is no config found at the config path
#[cfg(feature = "generate_config")]
pub fn generate_config(css_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(css_dir.join(Path::new("hljs")))?;

    let css_dark = include_bytes!("../example/css/github-markdown-dark.css");
    fs::File::create(css_dir.join(Path::new("github-markdown-dark.css")))?.write_all(css_dark)?;

    let css_light = include_bytes!("../example/css/github-markdown-light.css");
    fs::File::create(css_dir.join(Path::new("github-markdown-light.css")))?.write_all(css_light)?;

    let css_dark_hljs = include_bytes!("../example/css/hljs/github-dark.css");
    fs::File::create(css_dir.join(Path::new("hljs/github-dark.css")))?.write_all(css_dark_hljs)?;

    let css_light_hljs = include_bytes!("../example/css/hljs/github-light.css");
    fs::File::create(css_dir.join(Path::new("hljs/github-light.css")))?
        .write_all(css_light_hljs)?;

    Ok(())
}
