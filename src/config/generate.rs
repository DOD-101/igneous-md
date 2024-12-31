//! Generation of config
use regex::Regex;
use std::{collections::HashSet, path::Path};

const NOTICE_HLJS: &str = r#"
Theme taken from: https://github.com/highlightjs/highlight.js

For License details see upstream.
"#;

const NOTICE: &str = r#"
Theme taken from: https://github.com/sindresorhus/github-markdown-css/

For License details see upstream.

Changes:
- Hex color values replaced with variables
- Add 32px margin to body
"#;

pub async fn generate_config_files(css_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const HLJS_CSS_VERSION: &str = "11.11.1";

    const GH_CSS_VERSION: &str = "5.8.1";

    let request_client = reqwest::Client::default();

    let gh_dark_css = request_client.get(format!("https://raw.githubusercontent.com/sindresorhus/github-markdown-css/refs/tags/v{GH_CSS_VERSION}/github-markdown-dark.css")).send();
    let gh_dark_hljs_css = request_client.get(format!("https://raw.githubusercontent.com/highlightjs/highlight.js/refs/tags/{HLJS_CSS_VERSION}/src/styles/github-dark.css")).send();

    let gh_light_css = request_client.get(format!("https://raw.githubusercontent.com/sindresorhus/github-markdown-css/refs/tags/v{GH_CSS_VERSION}/github-markdown-light.css")).send();
    let gh_light_hljs_css = request_client.get(format!("https://raw.githubusercontent.com/highlightjs/highlight.js/refs/tags/{HLJS_CSS_VERSION}/src/styles/github-light.css")).send();

    // Wait for all responses and write files concurrently
    let (dark_res, dark_hljs_res, light_res, light_hljs_res) = tokio::try_join!(
        gh_dark_css.await?.text(),
        gh_dark_hljs_css.await?.text(),
        gh_light_css.await?.text(),
        gh_light_hljs_css.await?.text()
    )?;

    tokio::try_join!(
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-dark.css"),
                format!(
                    "@import url(\"./hljs/github-dark.css\");\n{}",
                    adjust_css(dark_res)
                ),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("hljs").join("github-dark.css"),
                format!("{}\n{}", NOTICE_HLJS, dark_hljs_res),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-light.css"),
                format!(
                    "@import url(\"./hljs/github-light.css\");\n{}",
                    adjust_css(light_res)
                ),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("hljs").join("github-light.css"),
                format!("{}\n{}", NOTICE_HLJS, light_hljs_res),
            )
            .await
        }
    )?;

    Ok(())
}

fn adjust_css(css: String) -> String {
    let hexes = find_hexes(&css);

    let new_css = replace_hexes(css, hexes.clone());

    format!(
        "/*{}*/\n{}\n{}\n{}",
        NOTICE,
        create_css_vars(hexes),
        ".markdown-body { margin: 32px !important; }",
        new_css
    )
}

fn replace_hexes(css: String, pairs: Vec<(String, String)>) -> String {
    pairs.iter().fold(css, |css, (hex, var)| {
        css.replace(hex, &format!("var({})", var))
    })
}

fn create_css_vars(pairs: Vec<(String, String)>) -> String {
    let mut result = ":root {\n".to_owned();

    for (col, var) in pairs {
        result.push_str(&format!("{}: {};\n", var, col))
    }

    result.push('}');

    result
}

fn find_hexes(text: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b").unwrap();

    // Find all matches in the text and collect them into a Vec
    let matches: Vec<String> = re
        .find_iter(text) // Iterate over all matches
        .map(|mat| mat.as_str().to_string()) // Convert matched strings to String
        .collect();

    let hexes: HashSet<String> = matches.into_iter().collect();

    hexes
        .into_iter()
        .enumerate()
        .map(|(index, hex)| (hex, format!("--color-{}", index)))
        .collect()
}
