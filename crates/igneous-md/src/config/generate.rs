//! Generation of the config. Disabled when compiled without the `generate_config` feature.
//!
//! This is the only part of the application that requires an internet connection.
use itertools::Itertools;
use regex::Regex;
use std::path::Path;

/// Copyright notice for highlight.js files
const NOTICE_HLJS: &str = r#"
Theme taken from: https://github.com/highlightjs/highlight.js

For License details see upstream.
"#;

/// Copyright notice for github-markdown-css files
const NOTICE: &str = r#"
Theme taken from: https://github.com/sindresorhus/github-markdown-css/

For License details see upstream.

Changes:
- Hex color values replaced with variables
- Add 32px margin to body
"#;

/// Responsible for generating the config files and writing them to disk.
///
/// The steps are as follows (not necessarily in order):
///
/// 1. Get the files from GitHub
///
/// 2. Adjust the css. See [adjust_css].
///
/// 3. Save to disk
///
/// For more information on different steps see the individual functions.
///
///
/// The function is async to speed up the fetching of the remote files and to allow concurrently writing
/// to disk.
pub async fn generate_config_files(css_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (dark_res, dark_hljs_res, light_res, light_hljs_res) = fetch_config_files().await?;

    tokio::try_join!(
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-dark.css"),
                adjust_css(dark_res.clone(), "github-dark.css", false),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-dark-centered.css"),
                adjust_css(dark_res.clone(), "github-dark.css", true),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("hljs").join("github-dark.css"),
                format!("/*{}*/\n{}", NOTICE_HLJS, dark_hljs_res),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-light.css"),
                adjust_css(light_res.clone(), "github-light.css", false),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("github-markdown-light-centered.css"),
                adjust_css(light_res.clone(), "github-light.css", true),
            )
            .await
        },
        async {
            tokio::fs::write(
                css_dir.join("hljs").join("github-light.css"),
                format!("/*{}*/\n{}", NOTICE_HLJS, light_hljs_res),
            )
            .await
        }
    )?;

    Ok(())
}

/// Fetch the base styles sheets from GitHub
///
/// The style sheets are specifically for the GitHub dark and light themes.
///
/// The tuples fields are: (dark, dark_hljs, light, light_hljs)
async fn fetch_config_files() -> reqwest::Result<(String, String, String, String)> {
    const HLJS_CSS_VERSION: &str = "11.11.1";

    const GH_CSS_VERSION: &str = "5.8.1";

    let request_client = reqwest::Client::default();

    let gh_dark_css = request_client.get(format!("https://raw.githubusercontent.com/sindresorhus/github-markdown-css/refs/tags/v{GH_CSS_VERSION}/github-markdown-dark.css")).send();
    let gh_dark_hljs_css = request_client.get(format!("https://raw.githubusercontent.com/highlightjs/highlight.js/refs/tags/{HLJS_CSS_VERSION}/src/styles/github-dark.css")).send();

    let gh_light_css = request_client.get(format!("https://raw.githubusercontent.com/sindresorhus/github-markdown-css/refs/tags/v{GH_CSS_VERSION}/github-markdown-light.css")).send();
    let gh_light_hljs_css = request_client.get(format!("https://raw.githubusercontent.com/highlightjs/highlight.js/refs/tags/{HLJS_CSS_VERSION}/src/styles/github.css")).send();

    // Wait for all responses and write files concurrently
    tokio::try_join!(
        gh_dark_css.await?.text(),
        gh_dark_hljs_css.await?.text(),
        gh_light_css.await?.text(),
        gh_light_hljs_css.await?.text()
    )
}

/// Adjust the given css String
///
/// Steps:
///
/// 1. Add Copyright notice
///
/// 2. Replace the hex color codes with css variables
///
/// 3. Add some custom additional styling
///
/// For more information see the individual functions.
fn adjust_css(css: String, hljs: &str, center: bool) -> String {
    let hexes = find_hexes(&css);

    let new_css = replace_hexes(css, hexes.clone());

    let mut aditional_styles = ".markdown-body { padding: 32px !important; ".to_string();

    if center {
        aditional_styles.push_str("max-width: 830px !important; margin: auto !important;");
    }

    aditional_styles.push('}');

    let css_vars = create_css_vars(hexes);

    format!(
        r#"
/*{NOTICE}*/
@import url(\"./hljs/{hljs}\");
{css_vars}
{aditional_styles}
{new_css}"#,
    )
}

/// Create the css for the variables
///
/// The [Vec<(String, String)>] format is the same as for [replace_hexes].
fn create_css_vars(pairs: Vec<(String, String)>) -> String {
    let mut result = ":root {\n".to_owned();

    for (col, var) in pairs {
        result.push_str(&format!("{}: {};\n", var, col))
    }

    result.push('}');

    result
}

/// Replace all hex color codes with the corresponding variable.
///
/// In the [Vec<(String, String)>], the first value is the hex color value to replace and the second is the
/// variable name to replace it with.
fn replace_hexes(css: String, pairs: Vec<(String, String)>) -> String {
    pairs.iter().fold(css, |css, (hex, var)| {
        css.replace(hex, &format!("var({})", var))
    })
}

/// Function used to find hex color values in css
///
/// It will find all valid hex color values so:
///
/// ```css
/// element {
///     color: #fff;
///     color: #ffffff;
///     color: #ffffffff; /* With alpha channel */
/// }
/// ```
///
/// Would all be detected.
///
/// The returned [Vec<(String, String)>] format is the same as for [replace_hexes].
fn find_hexes(text: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"#([0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b")
        .expect("Regex is hard-coded.");

    re.find_iter(text) // Iterate over all matches
        .map(|hex| hex.as_str().to_string())
        .unique()
        .enumerate()
        .map(|(index, hex)| (hex, format!("--color-{}", index)))
        .collect()
}

#[cfg(test)]
mod test {
    use super::{create_css_vars, fetch_config_files, find_hexes, replace_hexes};

    fn input_css() -> &'static str {
        r#"
        .class {
            background-color: #f00;
        }

        #id {
            color: #fe00ff;
            background: linear-gradient(#87ceeb, #4682b4);
        }

        el {
            border: 1px solid #ff1100ee;
            background-color: #f00; /* test to make sure duplicates are filtered out properly */
        }
        "#
    }

    fn output_pairs() -> Vec<(String, String)> {
        vec![
            ("#f00".to_string(), "--color-0".to_string()),
            ("#fe00ff".to_string(), "--color-1".to_string()),
            ("#87ceeb".to_string(), "--color-2".to_string()),
            ("#4682b4".to_string(), "--color-3".to_string()),
            ("#ff1100ee".to_string(), "--color-4".to_string()),
        ]
    }

    #[test]
    fn find_hexes_test() {
        let input = input_css();

        assert_eq!(find_hexes(input), output_pairs());
    }

    #[test]
    fn replace_hexes_test() {
        let input = input_css();

        // NOTE: Don't forget to update this when changing input_css()
        let expected_output = r#"
        .class {
            background-color: var(--color-0);
        }

        #id {
            color: var(--color-1);
            background: linear-gradient(var(--color-2), var(--color-3));
        }

        el {
            border: 1px solid var(--color-4);
            background-color: var(--color-0); /* test to make sure duplicates are filtered out properly */
        }
        "#;

        assert_eq!(
            replace_hexes(input.to_string(), output_pairs()),
            expected_output
        )
    }

    #[test]
    fn create_css_vars_test() {
        let pairs = output_pairs();
        let expected_output = format!(
            r#":root {{
{}: {};
{}: {};
{}: {};
{}: {};
{}: {};
}}"#,
            pairs[0].1,
            pairs[0].0,
            pairs[1].1,
            pairs[1].0,
            pairs[2].1,
            pairs[2].0,
            pairs[3].1,
            pairs[3].0,
            pairs[4].1,
            pairs[4].0,
        );
        assert_eq!(create_css_vars(output_pairs()), expected_output)
    }

    #[test]
    #[ignore = "only run on change of config file version."]
    fn fetch_errors() {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime.");
        let contents = rt
            .block_on(fetch_config_files())
            .expect("Failed to fetch files from GitHub. Do you have network access?");

        // Test against 404 errors
        assert!(!contents.0.starts_with("404: Not Found"));
        assert!(!contents.1.starts_with("404: Not Found"));
        assert!(!contents.2.starts_with("404: Not Found"));
        assert!(!contents.3.starts_with("404: Not Found"));

        // Test to make sure the files are long enough, anything shorter would indicate an error
        assert!(contents.0.len() > 100);
        assert!(contents.1.len() > 100);
        assert!(contents.2.len() > 100);
        assert!(contents.3.len() > 100);
    }
}
