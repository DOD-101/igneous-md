//! The conversion logic from md to HTML. See: [md_to_html]
//!
//! We also need to do some post processing [post_process_html] to make the resulting markdown work
//! for our application.
use kuchikiki::traits::*;
use markdown::{to_html_with_options, Options};

/// The actual conversion from md to HTML
///
/// Uses  [post_process_html] to adjust the HTML before returning it
pub fn md_to_html(md: &str) -> String {
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

    post_process_html(
        to_html_with_options(md, &markdown_options).expect("See docs of to_html_with_options."),
    )
}

/// Post process the given html, doing the following:
///
/// 1. Adding the missing classes for taks-lists
///
/// 2. Adjusts internal`.md` links to conform to the API format.
fn post_process_html(html: String) -> String {
    // Parse the HTML string into a DOM tree
    let document = kuchikiki::parse_html().one(html);

    // Add missing classes to task-lists
    //
    // These are not part of the gfm spec, hence not added by the converter.
    let checkboxes = document
        .select("li>p>input[type=\"checkbox\"]")
        .expect("Selector is hard-coded.");

    for checkbox in checkboxes {
        let checkbox = checkbox.as_node();
        let li = checkbox
            .parent()
            .and_then(|parent| parent.parent())
            .expect("The selector determines that these exist");
        let ul = li
            .parent()
            .expect("The selector determines that these exist");

        if let Some(checkbox_data) = checkbox.as_element() {
            let mut attributes = checkbox_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "task-list-item-checkbox".to_string());
        }

        if let Some(li_data) = li.as_element() {
            let mut attributes = li_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "task-list-item".to_string());
        }

        if let Some(ul_data) = ul.as_element() {
            let mut attributes = ul_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "contains-task-list".to_string());
        }
    }

    // Adjust markdown links to API format
    let links = document
        .select(r#"a[href$=".md"]:not([href^="http"])"#)
        .expect("Selector is hard-coded.");

    for link in links {
        let mut attributes = link.attributes.borrow_mut();

        let href = attributes
            .get_mut("href")
            .cloned()
            .expect("The selector determines that this exists");

        attributes.insert("href".to_string(), format!("/?path={}", href));
    }

    // Serialize the modified DOM back to HTML
    let mut output = Vec::new();
    document
        .serialize(&mut output)
        .expect("Serialization should never fail. All we did was add some classes.");
    String::from_utf8(output).expect("Converting to valid output should never fail.")
}

/// Returns the initial html, for when a client connects for the first time
///
/// This is mainly used to return a valid HTML document and load the required JS files.
pub fn initial_html(css: &str, body: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <title>Igneous-md</title>
        <script src="./src/highlight.min.js"></script>
        <script src="./src/main.js" defer></script>
        <link id="md-stylesheet" rel="stylesheet" href="{}" />
    </head>
    <body class="markdown-body" id="body">
    {}
    </body>
    </html>
    "#,
        css, body
    )
}

#[cfg(test)]
mod test {
    use super::md_to_html;
    use kuchikiki::traits::*;

    #[test]
    fn links() {
        let md_input = [
            "[](https://test.md)",
            "[](http://test.md)",
            "[](/test.md)",
            "[](./test.md)",
            "[](../test.md)",
        ];

        let html_ouput = md_input.map(md_to_html);

        let mut hrefs = html_ouput.into_iter().map(|h| {
            let document = kuchikiki::parse_html().one(h);

            // Adjust markdown links to API format
            let link = document
                .select_first(r#"a"#)
                .expect("Selector is hard-coded.");

            let attributes = link.attributes.borrow_mut();

            attributes.get("href").unwrap().to_string()
        });

        assert_eq!(hrefs.next(), Some("https://test.md".to_string()));
        assert_eq!(hrefs.next(), Some("http://test.md".to_string()));
        assert_eq!(hrefs.next(), Some("/?path=/test.md".to_string()));
        assert_eq!(hrefs.next(), Some("/?path=./test.md".to_string()));
        assert_eq!(hrefs.next(), Some("/?path=../test.md".to_string()));
        assert_eq!(hrefs.next(), None);
    }
}
