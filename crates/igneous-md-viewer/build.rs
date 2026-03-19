use std::{
    env,
    fs::{self, read_to_string},
    path::PathBuf,
    process::Command,
};

use askama::Template;

#[derive(Template)]
#[template(path = "index.html.askama", escape = "none")]
struct Index<'a> {
    mainjs: &'a str,
    highlightjs: &'a str,
    mathjaxjs: &'a str,
}

fn main() {
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

    let mut contents = String::new();

    Index {
        mainjs: read_to_string(dist_location).unwrap().as_str(),
        highlightjs: include_str!("./src/highlight.min.js"),
        mathjaxjs: include_str!("./src/mathjaxV4.js"),
    }
    .render_into(&mut contents)
    .unwrap();

    fs::write(out_dir.join("index.html"), contents).unwrap();
}
