use home::home_dir;
use serde::Deserialize;
use std::fs::read_to_string;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub initial_css: String,
    pub css_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_css: "github-markdown-dark.css".to_string(),
            css_dir: format!("{}css/", config_path()),
        }
    }
}

impl Config {
    pub fn read_config() -> Self {
        let path = format!("{}/config.toml", config_path());

        match read_to_string(path) {
            Ok(config) => {
                let config_result: Result<Config, _> = toml::from_str(&config);

                match config_result {
                    Ok(config) => config,
                    Err(e) => {
                        println!("{:?}", e);
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
// TODO: Update path to match future project title
pub fn config_path() -> &'static String {
    static CONFIG_PATH: OnceLock<String> = OnceLock::new();
    CONFIG_PATH.get_or_init(|| {
        format!(
            "{}/.config/quick-md/",
            home_dir()
                .expect("Couldn't find the home dir!")
                .to_string_lossy()
        )
    })
}
