//! The viewer component of igneous_md
//!
//! This binary just launches the viewer.
//!
//! It's useful for when you accidentally closed the viewer, but don't want to restart the whole
//! server.
use clap::Parser;
use gtk::glib::BoolError;
use igneous_md_viewer::Viewer;

// TODO: It might be nice to be able to pass a default css you want to use. This could be done via
// URL arguments. But this is primarily a change that would need to happen in the server

fn main() -> Result<(), BoolError> {
    let cli = Cli::parse();

    let viewer = Viewer::new(format!("localhost:{}/?path={}", cli.port, cli.path));

    viewer.start()?;

    Ok(())
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(default_value = "README.md")]
    pub path: String,
    #[arg(default_value = "2323")]
    pub port: u16,
}
