use std::{
    collections::HashMap,
    env,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    process::Command,
};

use askama::Template;

const VENDOR_ENV: &str = include_str!("../../vendor.env");

#[derive(Template)]
#[template(path = "index.html.askama", escape = "none")]
struct Index<'a> {
    mainjs: &'a str,
    highlightjs: &'a str,
    mathjaxjs: &'a str,
}

fn parse_env(input: &str) -> HashMap<&str, &str> {
    input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim(), value.trim()))
        })
        .collect()
}

fn download(url: &str, dest: &Path) {
    assert!(
        Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--fail",
                "--output",
                dest.display().to_string().as_str(),
                url,
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success(),
        "Failed to download {url}"
    );
}

/// Resolve a JS dependency.
///
/// If a vendored copy exists in `src/`, use it. Otherwise, if `IGNEOUS_VENDOR_ONLY` is set, fail.
/// Otherwise, download.
fn resolve_js(vendored: &Path, url: &str, out_dir: &Path) -> PathBuf {
    let vendor_only = env::var("IGNEOUS_VENDOR_ONLY").is_ok();

    if vendored.exists() {
        return vendored.to_path_buf();
    }

    if vendor_only {
        panic!(
            "IGNEOUS_VENDOR_ONLY is set but {} is missing.\n\
             Download it from {url} and place it in src/",
            vendored.display()
        );
    }

    let dest = out_dir.join(vendored.file_name().unwrap());
    download(url, &dest);
    dest
}

fn main() {
    let deps = parse_env(VENDOR_ENV);
    let highlight_url = deps["HIGHLIGHT_JS_URL"];
    let mathjax_url = deps["MATHJAX_URL"];

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let dist_location = out_dir.join("./dist.js");

    assert!(Command::new("esbuild")
        .args([
            "--bundle",
            "./src/main.js",
            format!("--outfile={}", dist_location.display()).as_str(),
            "--minify"
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());

    let src_dir = PathBuf::from("./src");

    let highlight_path = resolve_js(&src_dir.join("highlight.min.js"), highlight_url, &out_dir);
    let mathjax_path = resolve_js(&src_dir.join("tex-mml-svg.js"), mathjax_url, &out_dir);

    let mut contents = String::new();

    Index {
        mainjs: read_to_string(&dist_location).unwrap().as_str(),
        highlightjs: read_to_string(&highlight_path).unwrap().as_str(),
        mathjaxjs: read_to_string(&mathjax_path).unwrap().as_str(),
    }
    .render_into(&mut contents)
    .unwrap();

    fs::write(out_dir.join("index.html"), contents).unwrap();
}
