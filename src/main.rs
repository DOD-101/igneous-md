use clap::Parser;
use rouille::{match_assets, router, start_server, Response};
use std::{fs, process::exit, thread};
use web_view::*;

mod config;
mod handlers;

use config::*;

fn main() {
    let args = Args::parse();
    let config = Config::read_config();

    // address of the server
    let address = args.address.to_owned();

    let initial_css = match args.css.to_owned() {
        Some(css) => css,
        None => config.initial_css.to_owned(),
    };

    // localhost:port/path/to/file
    let md_url = format!("{}/{}", address, args.path);

    println!("Starting live-reload server on {}", md_url);

    if !args.no_viewer {
        thread::spawn(move || client(&md_url));
    }

    let all_css: Vec<fs::DirEntry> = match fs::read_dir(&config.css_dir) {
        Ok(dir) => dir
            .filter_map(|css| match css {
                Ok(entry) => {
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
            println!("Failed to read css dir: {}", &config.css_dir);
            exit(1)
        }
    };

    start_server(address, move |request| {
        if !args.quiet {
            println!("SERVER: Got request. With url: {:?}", request.url());
        }

        router!(request,
            (GET) (/api/get-css-path) => {handlers::get_css_path(request, &all_css)},
            _ => {
                if request.url().ends_with(".md") {
                        return handlers::get_md(request, &args, &initial_css);
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
                    return handlers::get_css(request, &config);
                }

                if !args.quiet {
                     println!("Refusing");
                }

                Response::html("404 Error").with_status_code(404)
            }
        )
    });
}

fn client(addr: &str) {
    println!("Starting client on {addr}");
    if web_view::builder()
        .title("igneous-md")
        .content(Content::Url(format!("http://{}", addr)))
        .size(320, 480)
        .resizable(true)
        .debug(false)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .is_err()
    {
        println!("Couldn't start client")
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    css: Option<String>,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    #[arg(short, long, default_value = "false")]
    quiet: bool,
    #[arg(short, long, default_value = "localhost:2323")]
    address: String,
}
