use kuchikiki::traits::*;
use markdown::{to_html_with_options, Options};

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

    post_process_html(to_html_with_options(md, &markdown_options).unwrap())
}

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

    // Adjust markdown links to to api format
    let links = document
        .select(r#"a[href$=".md"]:not([href*="https:"])"#)
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
pub fn initial_html(css: &str, body: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset=\"utf-8\"/>
        <title>My Project</title>
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
