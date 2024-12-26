//! Paths for the application
use home::home_dir;
use std::sync::OnceLock;

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
