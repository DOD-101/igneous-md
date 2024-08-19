use clap::Parser;
mod config;
use config::*;
use markdown::{to_html_with_options, Options};
use rouille::{match_assets, router, start_server, Request, Response};
use std::collections::HashMap;
use std::fs::DirEntry;
use std::process::exit;
use std::{fs, thread};
use web_view::*;

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

    let all_css: Vec<DirEntry> = match fs::read_dir(&config.css_dir) {
        Ok(dir) => dir
            .filter_map(|css| match css {
                Ok(entry) => {
                    if !args.quiet {
                        println!("CSS Option: {:#?}", entry);
                    }
                    Some(entry)
                }
                Err(error) => {
                    println!("Could not read entries in css folder: {:#?}", error);
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
            (GET) (/api/get-css) => {get_css_path(request, &all_css)},
            _ => {
                if request.url().ends_with(".md") {
                        return get_md(request, &args, &initial_css);
                }

                {
                    let response = match_assets(request, ".");

                    if response.is_success() {
                        return response;
                    }
                }

                if request.url().ends_with(".css") {
                    return get_css(request, &config);
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
        // TODO: Update title to match future project title
        .title("My Project")
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

fn inital_html(css: &str, body: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
    <meta charset=\"utf-8\"/>
    <title>My Project</title>
    <script src="./main.js" defer></script>
    <link id="stylesheet" rel="stylesheet" href="{}" />
    </head>
    <body class="markdown-body" id="body">
    {}
    </body>
    </html>
    "#,
        css, body
    )
}

fn get_css_path(request: &Request, all_css: &[DirEntry]) -> Response {
    let arguments =
        match serde_urlencoded::from_str::<HashMap<String, String>>(request.raw_query_string()) {
            Ok(args) if args.contains_key("n") => args.get("n").unwrap().to_owned(),
            _ => return Response::html("404 Error: Invalid URL-Parameters").with_status_code(404),
        };

    let index: usize = match arguments.parse::<usize>() {
        Ok(num) => num % all_css.len(),
        Err(_) => return Response::html("404 Error: Invalid URL-Parameters").with_status_code(404),
    };

    Response::text(all_css[index].path().file_name().unwrap().to_string_lossy())
}

fn get_css(request: &Request, config: &Config) -> Response {
    let response = match_assets(request, &config.css_dir);

    if response.is_success() {
        return response;
    }

    println!("Failed to match css");
    Response::html("404 Error").with_status_code(404)
}

fn get_md(request: &Request, args: &Args, initial_css: &str) -> Response {
    let md = fs::read_to_string(format!(".{}", request.url()));

    if md.is_err() {
        return Response::html("404 Error").with_status_code(404);
    }

    let mut html = to_html_with_options(&md.unwrap(), &Options::gfm()).unwrap();

    // if no parameters are passed send full html
    if request.raw_query_string().is_empty() {
        html = inital_html(initial_css, &html);
    }

    if args.verbose {
        println!("SERVER: Sending: {html}");
    }

    Response::html(html)
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
