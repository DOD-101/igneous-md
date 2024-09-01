//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md --path path/to/file.md
//! ```
//! For more information see the usage docs.
//!
use clap::Parser;
use gtk::{prelude::*, Window, WindowType};
use rouille::{match_assets, router, start_server, Response};
use std::{fs, process::exit, thread};
use webkit2gtk::{WebView, WebViewExt};

mod handlers;

mod paths;

use paths::{config_path, css_path};

fn main() {
    let args = Args::parse();

    // address of the server
    let address = args.address.to_owned();

    let mut all_css: Vec<fs::DirEntry> = match fs::read_dir(css_path()) {
        Ok(dir) => dir
            .filter_map(|css| match css {
                Ok(entry) => {
                    if entry.path().is_dir() {
                        return None;
                    }

                    if !args.quiet {
                        println!("CSS Option: {:#?}", entry);
                    }
                    Some(entry)
                }
                Err(error) => {
                    println!("Could not read entry in css folder: {:#?}", error);
                    None
                }
            })
            .collect(),
        Err(_) => {
            println!("Failed to read css dir: {}", css_path());
            exit(1)
        }
    };

    all_css.sort_by_key(|a| a.file_name());

    let initial_css = match args.css.to_owned() {
        Some(css) => css,
        None => all_css[0].file_name().to_string_lossy().to_string(),
    };

    // localhost:port/path/to/file
    let md_url = format!("{}/{}", address, args.path);

    println!("Starting live-reload server on {}", md_url);

    if args.browser && open::that_detached(&md_url).is_err() {
        println!("Failed to open browser");
    }

    if !args.no_viewer {
        thread::spawn(move || client(&md_url));
    }

    start_server(address, move |request| {
        if !args.quiet {
            println!("SERVER: Got request. With url: {:?}", request.url());
        }
        router!(request,
            (GET) (/api/get-css-path) => {handlers::get_css_path(request, &all_css)},
            (POST) (/api/post-html) => {handlers::save_html(request, &Args::parse())},
            // TODO: Need to rework the handlers to not need the args. Since they are primarly
            // being used to check for --verbose anyway. And logging should be moved out of these
            // functions, if possible.
            (GET) (/ws) => {handlers::upgrade_connection(request, Args::parse())},
            _ => {
                if request.url().ends_with(".md") {
                        return handlers::get_inital_md(request, &args, &initial_css);
                }

                {
                    // Match any assets in the current dir
                    let response = match_assets(request, ".");

                    if response.is_success() {
                        return response;
                    }
                }

                // if that fails check if it is a .css, in which case it is probably
                // in the css dir
                // TODO: This creates an issue where we might accidentally get a different
                // stylesheet than the user wants, if they have one with the same name in
                // the current working dir
                if request.url().ends_with(".css") {
                    return handlers::get_css(request);
                }

                if !args.quiet {
                     println!("Refusing");
                }

                Response::html("404 Error").with_status_code(404)
            }
        )
    });
}

/// Starts the markdown viewer
fn client(addr: &str) {
    println!("Starting client on {addr}");
    gtk::init().unwrap();

    let window = Window::new(WindowType::Toplevel);
    window.set_title("igneous-md viewer");
    window.set_default_size(800, 600);

    let view = WebView::new();
    view.load_uri(&format!("http://{addr}"));

    window.add(&view);

    window.show_all();

    gtk::main()
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    /// Path to markdown file
    #[arg(short, long)]
    path: String,
    /// Path to stylesheet
    #[arg(short, long, value_name = "PATH")]
    css: Option<String>,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "false")]
    quiet: bool,
    #[arg(short, long, default_value = "localhost:2323")]
    address: String,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    browser: bool,
}
