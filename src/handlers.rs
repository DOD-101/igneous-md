/// The module has the functions that handle the different incoming requests
use markdown::{to_html_with_options, Options};
use rouille::{match_assets, Request, Response};
use std::{collections::HashMap, fs};

use crate::config::Config;
use crate::Args;

pub fn get_css_path(request: &Request, all_css: &[fs::DirEntry]) -> Response {
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

pub fn get_css(request: &Request, config: &Config) -> Response {
    let response = match_assets(request, &config.css_dir);

    if response.is_success() {
        return response;
    }

    println!("Failed to match css");
    Response::html("404 Error").with_status_code(404)
}

pub fn get_md(request: &Request, args: &Args, initial_css: &str) -> Response {
    let md = fs::read_to_string(format!(".{}", request.url()));

    if md.is_err() {
        return Response::html("404 Error").with_status_code(404);
    }

    let markdown_options = Options {
        parse: markdown::ParseOptions {
            constructs: markdown::Constructs {
                html_flow: true,
                html_text: true,
                definition: true,
                ..markdown::Constructs::gfm()
            },
            ..markdown::ParseOptions::gfm()
        },
        compile: markdown::CompileOptions {
            allow_dangerous_html: true,
            ..markdown::CompileOptions::gfm()
        },
    };

    let mut html = to_html_with_options(&md.unwrap(), &markdown_options).unwrap();

    // if no parameters are passed send full html
    if request.raw_query_string().is_empty() {
        html = initial_html(initial_css, &html);
    }

    if args.verbose {
        println!("SERVER: Sending: {html}");
    }

    Response::html(html)
}

fn initial_html(css: &str, body: &str) -> String {
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
