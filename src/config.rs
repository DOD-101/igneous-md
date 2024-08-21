//! Configuration related functionality
use home::home_dir;
use serde::Deserialize;
use std::{fs, sync::OnceLock};

/// Structure of the config containing all fields
#[derive(Debug, Deserialize)]
pub struct Config {
    pub initial_css: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_css: "github-markdown-dark.css".to_string(),
        }
    }
}

impl Config {
    /// Attempts to read the configuration file, if this fails it returns the default values
    pub fn read_config() -> Self {
        let path = format!("{}/config.toml", config_path());

        match fs::read_to_string(path) {
            Ok(config) => {
                let config_result: Result<Config, _> = toml::from_str(&config);

                match config_result {
                    Ok(config) => config,
                    Err(e) => {
                        println!("Error in config {:?}", e);
                        Config::default()
                    }
                }
            }
            Err(_) => {
                println!("Could not find config file. Using defaults.");
                println!("Consider creating one at {}config.toml", config_path());
                Config::default()
            }
        }
    }
}

/// Returns the config path for the application
pub fn config_path() -> &'static String {
    static CONFIG_PATH: OnceLock<String> = OnceLock::new();
    CONFIG_PATH.get_or_init(|| {
        format!(
            "{}/.config/igneous-md/",
            home_dir()
                .expect("Couldn't find the home dir!")
                .to_string_lossy()
        )
    })
}

/// Returns the path to the css files
pub fn css_path() -> &'static String {
    static CSS_PATH: OnceLock<String> = OnceLock::new();
    CSS_PATH.get_or_init(|| format!("{}css/", config_path()))
}
