//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md --path path/to/file.md
//! ```
//! For more information see the usage docs.
//!
use std::io::Write;
use std::{fs, io, path::Path, process::exit, thread};

use clap::Parser;
use gtk::{prelude::*, Window, WindowType};
use rouille::{match_assets, router, start_server, Response};
use simple_logger::SimpleLogger;
use webkit2gtk::{CacheModel, WebContext, WebContextExt, WebView, WebViewExt};

mod handlers;
mod paths;

use paths::{config_path, default_css_path};

fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();

    log::set_max_level(args.log_level);

    // address of the server
    let address = args.address.to_owned();

    // dir of the css files
    let css_dir = args.css_dir.clone();

    // all css files in that dir
    let all_css = get_all_css(&css_dir);

    // the first css file to load
    let initial_css = match args.css.to_owned() {
        // The path given by the user
        Some(css) => css,
        // If there are no css paths
        None if all_css.is_empty() => String::new(),
        // Since we know there is at least one path we can safely index
        None => all_css[0].file_name().to_string_lossy().to_string(),
    };

    // The url of the md file, in the format:
    // localhost:port/path/to/file
    let md_url = format!("{}/{}", address, args.path);

    log::info!("Starting live-reload server on {}", md_url);

    if args.browser && open::that_detached(&md_url).is_err() {
        log::warn!("Failed to open browser");
    }

    if !args.no_viewer {
        thread::spawn(move || client(&md_url));
    }

    let hljs_src = include_str!("./highlight.min.js");
    let js_src = include_str!("./main.js");

    start_server(address, move |request| {
        log::info!("SERVER: Got request. With url: {:?}", request.url());
        router!(request,
            (GET) ["/src/main.js"] => {Response::from_data("text/javascript", js_src)},
            (GET) ["/src/highlight.min.js"] => {Response::from_data("text/javascript", hljs_src)},
            (GET) ["/api/get-css-path"] => {handlers::get_css_path(request, &all_css)},
            (GET) ["/css/{_path}", _path: String] => {handlers::get_css(request, &css_dir)},
            (GET) ["/css/hljs/{_path}", _path: String] => {handlers::get_css(request, &css_dir)},
            (POST) ["/api/post-html"] => {handlers::save_html(request)},
            (GET) ["/ws"] => {handlers::upgrade_connection(request)},
            _ => {
                if request.url().ends_with(".md") {
                        return handlers::get_inital_md(request, &initial_css);
                }

                {
                    // Match any assets in the current dir
                    let response = match_assets(request, ".");

                    if response.is_success() {
                        return response;
                    }
                }

                log::info!("Got invalid request: {:?}", request.url());
                Response::html("404 Error").with_status_code(404)
            }
        )
    });
}

/// Starts the markdown viewer
fn client(addr: &str) {
    log::info!("Starting client on {addr}");
    if gtk::init().is_err() {
        log::error!("Failed to init gtk. Needed for viewer.");
        exit(1)
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("igneous-md viewer");
    window.set_default_size(800, 600);

    let context = WebContext::default().unwrap();
    context.set_cache_model(CacheModel::DocumentViewer);
    context.clear_cache();

    let view = WebView::with_context(&context);

    view.load_uri(&format!("http://{addr}"));

    window.add(&view);

    window.show_all();

    gtk::main()
}

/// Gets all of the css files in the given dir
/// Will not look in any sub-dirs
///
/// If the `css_dir` is the default path then this function will generate the missing config. If this
/// is not the case then it will exit.
fn get_all_css(css_dir: &str) -> Vec<fs::DirEntry> {
    let mut all_css = match fs::read_dir(css_dir) {
        Ok(dir) => dir
            .filter_map(|css| match css {
                Ok(entry) => {
                    if entry.path().is_dir() {
                        return None;
                    }

                    log::info!("CSS Option: {:#?}", entry);

                    Some(entry)
                }
                Err(error) => {
                    log::warn!("Could not read entry in css folder: {:#?}", error);
                    None
                }
            })
            .collect(),
        Err(_) => {
            log::warn!("Failed to read css dir: {}", css_dir);
            if css_dir == default_css_path() {
                log::info!("Generating default config dir contents.");
                if let Err(e) = generate_config(Path::new(css_dir)) {
                    log::error!("Failed to generate config. Error: {}", e);
                    exit(1);
                }
            } else {
                log::error!(
                    "Exiting due to custom css dir not being readable: {}",
                    css_dir
                );
                exit(1);
            }

            get_all_css(css_dir)
        }
    };

    all_css.sort_by_key(|a| a.file_name());

    all_css
}

/// Creates the css files on disk
///
/// Used if there is no config found at the config path
fn generate_config(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path.join(Path::new("hljs")))?;

    let css_dark = include_bytes!("../example/css/github-markdown-dark.css");
    fs::File::create(path.join(Path::new("github-markdown-dark.css")))?.write_all(css_dark)?;

    let css_light = include_bytes!("../example/css/github-markdown-light.css");
    fs::File::create(path.join(Path::new("github-markdown-light.css")))?.write_all(css_light)?;

    let css_dark_hljs = include_bytes!("../example/css/hljs/github-dark.css");
    fs::File::create(path.join(Path::new("hljs/github-dark.css")))?.write_all(css_dark_hljs)?;

    let css_light_hljs = include_bytes!("../example/css/hljs/github-light.css");
    fs::File::create(path.join(Path::new("hljs/github-light.css")))?.write_all(css_light_hljs)?;

    Ok(())
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    /// Path to markdown file
    #[arg(short, long)]
    path: String,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    css: Option<String>,
    /// Path to the css dir
    ///
    /// Defaults to config_dir/css, if that doesn't exist uses the example.
    #[arg(long, value_name = "PATH", default_value = &**default_css_path())]
    css_dir: String,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "Info")]
    log_level: log::LevelFilter,
    #[arg(short, long, default_value = "localhost:2323")]
    address: String,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    browser: bool,
}
