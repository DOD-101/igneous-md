use clap::Parser;
use markdown::{to_html_with_options, Options};
use rouille::{match_assets, start_server, Response};
use std::{fs, thread};
use web_view::*;

const ADDRESS: &str = "localhost:2323";

fn main() {
    let args = Args::parse();
    thread::spawn(move || client(&format!("{}/{}", ADDRESS, args.path)));

    let css = fs::read_to_string(args.css).unwrap();

    println!("Starting server on {}", ADDRESS);

    start_server(ADDRESS, move |request| {
        println!("SERVER: Got request. With url: {:?}", request.url());

        if request.url().ends_with(".md") {
            let md = fs::read_to_string(format!(".{}", request.url()));

            if md.is_err() {
                return Response::html("404 Error").with_status_code(404);
            }

            let mut html = to_html_with_options(&md.unwrap(), &Options::gfm()).unwrap();

            if !request.raw_query_string().is_empty() {
                html = create_html(&css, &html);
            }

            if args.verbose {
                println!("SERVER: Sending: {html}");
            }
            return Response::html(html);
        }

        {
            let respoonse = match_assets(request, ".");

            if respoonse.is_success() {
                return respoonse;
            }
        }
        println!("Refusing");
        Response::html("404 Error").with_status_code(404)
    });
}

fn client(addr: &str) {
    println!("Starting client on {addr}");
    if web_view::builder()
        .title("My Project")
        .content(Content::Url(format!("http://{}?init=true", addr)))
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

fn create_html(css: &str, body: &str) -> String {
    format!(
        r#"
    <html>
    <head>
    <meta charset=\"utf-8\"/>
    <title>My Project</title>
    <script src="./main.js" defer></script>
    <style>
    .markdown-body {{
        padding: 32px;
    }}
    {}</style>
    </head>
    <body class="markdown-body" id="body">
    {}
    </body>
    </html>
    "#,
        css, body
    )
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long, default_value = "github-markdown-dark.css")]
    css: String,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}
