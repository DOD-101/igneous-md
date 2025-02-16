//! The viewer component of igneous_md
//!
//! This binary just launches the viewer.
//!
//! It's useful for when you accidentally closed the viewer, but don't want to restart the whole
//! server.
use clap::Parser;
use gtk::glib::BoolError;
use igneous_md_viewer::Viewer;

fn main() -> Result<(), BoolError> {
    let cli = Cli::parse();

    let viewer = Viewer::new(format!(
        "localhost:{}/{}",
        cli.port,
        cli.css.map(|s| format!("?css={}", s)).unwrap_or_default()
    ));

    viewer.start()?;

    Ok(())
}

#[derive(Debug, Parser)]
/// Igneous-md viewer
///
/// Used to connect to an already running igneous-md server.
pub struct Cli {
    /// Port the server is running on
    #[arg(short, long, default_value = "2323")]
    pub port: u16,
    /// Path to the initial css to use
    #[arg(short, long)]
    pub css: Option<String>,
}
