use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
    vec::IntoIter,
};

use crate::bidirectional_cycle::{BiCyclable, BiCycle};

#[derive(Debug, Clone)]
pub struct Config {
    config_dir: PathBuf,
    css_dir: PathBuf,
    css_iter: Option<BiCycle<IntoIter<PathBuf>>>,
    css_paths: Vec<PathBuf>,
}

impl Config {
    pub fn new(config_dir: PathBuf) -> io::Result<Self> {
        let css_dir = config_dir.join("css");

        let mut config = Self {
            config_dir,
            css_dir,
            css_iter: None,
            css_paths: vec![],
        };

        config.update_css_paths()?;
        config.css_iter = Some(config.css_paths.clone().into_iter().bi_cycle());

        log::info!("{:?}", config.css_paths);

        Ok(config)
    }

    pub fn next_css(&mut self) -> Option<PathBuf> {
        if let Some(iter) = &mut self.css_iter {
            return iter.next().map(|p| {
                p.strip_prefix(self.config_dir.clone())
                    .unwrap()
                    .to_path_buf()
            });
        }
        None
    }

    pub fn previous_css(&mut self) -> Option<PathBuf> {
        if let Some(iter) = &mut self.css_iter {
            return iter.next_back().map(|p| {
                p.strip_prefix(self.config_dir.clone())
                    .unwrap()
                    .to_path_buf()
            });
        }
        None
    }

    pub fn get_css_dir(&self) -> &PathBuf {
        &self.css_dir
    }

    #[allow(dead_code)]
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Reads the css dir, updating self.css_paths
    pub fn update_css_paths(&mut self) -> io::Result<()> {
        let mut all_css: Vec<PathBuf> = match fs::read_dir(self.css_dir.as_path()) {
            Ok(dir) => dir
                .filter_map(|css| match css {
                    Ok(entry) => {
                        if entry.path().is_dir() {
                            return None;
                        }

                        log::info!("CSS Option: {:#?}", entry);

                        Some(entry.path())
                    }
                    Err(error) => {
                        log::warn!("Could not read entry in css folder: {:#?}", error);
                        None
                    }
                })
                .collect(),
            Err(e) => {
                log::warn!("Failed to read css dir: {}", self.css_dir.to_string_lossy());

                return Err(e);
            }
        };

        all_css.sort_by_key(|a| PathBuf::from(a.file_name().unwrap()));

        self.css_paths = all_css;

        Ok(())
    }
}

/// Creates the css files on disk
///
/// Used if there is no config found at the config path
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
