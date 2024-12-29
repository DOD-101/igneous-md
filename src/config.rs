use std::{fs, io, path::PathBuf, vec::IntoIter};

#[cfg(feature = "generate_config")]
use std::{io::Write, path::Path};

use crate::{
    bidirectional_cycle::{BiCyclable, BiCycle},
    paths::Paths,
};

#[derive(Debug, Clone)]
pub struct Config {
    config_dir: PathBuf,
    css_dir: PathBuf,
    css_iter: Option<BiCycle<IntoIter<PathBuf>>>,
    css_paths: Vec<PathBuf>,
    curent_css: Option<PathBuf>,
}

impl Config {
    pub fn new(paths: Paths) -> io::Result<Self> {
        let mut config = Self {
            config_dir: paths.get_config_dir(),
            css_dir: paths.get_css_dir(),
            css_iter: None,
            css_paths: vec![],
            curent_css: paths.get_default_css(),
        };

        config.update_css_paths()?;
        config.css_iter = Some(config.css_paths.clone().into_iter().bi_cycle());

        if !config.css_paths.is_empty() && config.curent_css.is_none() {
            config.next_css();
        }

        Ok(config)
    }

    pub fn next_css(&mut self) -> Option<PathBuf> {
        if let Some(iter) = &mut self.css_iter {
            let css = iter
                .next()
                .map(|p| PathBuf::from("/css").join(p.strip_prefix(self.css_dir.clone()).unwrap()));

            self.curent_css = css.clone();

            return css;
        }
        None
    }

    pub fn previous_css(&mut self) -> Option<PathBuf> {
        if let Some(iter) = &mut self.css_iter {
            let css = iter
                .next_back()
                .map(|p| PathBuf::from("/css").join(p.strip_prefix(self.css_dir.clone()).unwrap()));

            self.curent_css = css.clone();

            return css;
        }
        None
    }

    #[allow(dead_code)]
    pub fn current_css(&self) -> Option<PathBuf> {
        self.curent_css.clone()
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
        let mut all_css: Vec<PathBuf> = match fs::read_dir(self.css_dir.as_path()) {
            Ok(dir) => dir
                .filter_map(|css| match css {
                    Ok(entry) => {
                        if entry.path().is_dir() {
                            return None;
                        }

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

        log::info!("Updated css_paths: {:?}", self.css_paths);

        Ok(())
    }
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
