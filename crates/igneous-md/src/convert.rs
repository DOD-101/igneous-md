//! The conversion logic from md to HTML. See: [md_to_html]
//!
//! We also need to do some post processing [post_process_html] to make the resulting markdown work
//! for our application.
use kuchikiki::{traits::*, NodeRef};
use markdown::{to_html_with_options, Options};
use markup5ever::{interface::QualName, local_name, namespace_url, ns};
use regex::Regex;
use std::collections::{HashMap, VecDeque};

static ALERT_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"^\s*\[!(?i)(note|tip|important|warning|caution)\](\s*$|\s*\n.*$)"#)
        .expect("Regex is hard-coded.")
});

static SVGS: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(|| {
        HashMap::from([
            ("Note", include_str!("../../../assets/info-16.svg")),
            ("Tip", include_str!("../../../assets/light-bulb-16.svg")),
            ("Important", include_str!("../../../assets/report-16.svg")),
            ("Warning", include_str!("../../../assets/alert-16.svg")),
            ("Caution", include_str!("../../../assets/stop-16.svg")),
        ])
    });

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
///
/// 3. Adds GitHub-style alerts
fn post_process_html(html: String) -> String {
    // Parse the HTML string into a DOM tree
    let document = kuchikiki::parse_html().one(html);

    // --- Add missing classes to task-lists ---
    //
    // These are not part of the gfm spec, hence not added by the converter.
    let checkboxes = document
        .select("li input[type=\"checkbox\"]")
        .expect("Selector is hard-coded.");

    for checkbox in checkboxes {
        let checkbox = checkbox.as_node();
        let li = checkbox
            .ancestors()
            .find(|n| {
                n.as_element()
                    .expect("We know this is an element.")
                    .name
                    .local
                    .eq("li")
            })
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

    // --- Adjust markdown links to API format ---
    let links = document
        .select(r#"a[href$=".md"]:not([href^="http"])"#)
        .expect("Selector is hard-coded.");

    for link in links {
        let mut attributes = link.attributes.borrow_mut();

        let href = attributes
            .get_mut("href")
            .expect("The selector ensures the existence of an href with content.")
            .clone();

        attributes.insert("onclick", format!("return handle_redirect(\"{}\")", href));
    }

    // --- Add GitHub-style markdown alerts / highlight-notes
    let alerts_data: Vec<(kuchikiki::NodeDataRef<kuchikiki::ElementData>, String)> = document
        .select(r#"blockquote > p:first-child"#)
        .expect("Selector is hard-coded.")
        .filter_map(|a| {
            // We're getting the first text node of `a` to use in the regex
            if let Some(text) = a.as_node().first_child().map(|c| c.text_contents()) {
                // If the regex matches then return `a` and the `alert-type` we matched
                if let Some(Some(alert_type)) = ALERT_REGEX.captures(&text).map(|c| c.get(1)) {
                    return Some((a, alert_type.as_str().to_string()));
                }
            }

            None
        })
        .collect();

    for (a, alert_type) in alerts_data {
        let blockquote = a
            .as_node()
            .parent()
            .expect("The selector ensures the existence of the blockquote.");

        let mut blockquote_children: VecDeque<NodeRef> = blockquote
            .children()
            .filter(|c| c.text_contents() != "\n")
            .collect();

        // First p in the blockquote
        let first_p = blockquote_children
            .remove(0)
            .expect("The selector ensures the existence of the p element");

        let mut first_p_children = first_p.children();

        // Split the first "text" of the first p element to get any text in the line after the
        // alert title
        //
        // We need to do this since in a normal block quote the first line has no special meaning
        // and will therefore be put into the same `p` as the text on the second line, if there is
        // no empty line between the two.
        let content_after_title = first_p_children
            .next()
            .and_then(|c| {
                c.text_contents()
                    .split_once('\n')
                    .map(|(_, after)| after.to_owned())
            })
            .unwrap_or_default();

        // Make the div for the alert
        //
        // This replaces the blockquote
        let alert_container_node = kuchikiki::NodeRef::new_element(
            QualName::new(None, ns!(html), local_name!("div")),
            [(
                kuchikiki::ExpandedName::new(ns!(), local_name!("class")),
                kuchikiki::Attribute {
                    prefix: None,
                    value: format!(
                        "markdown-alert markdown-alert-{}",
                        alert_type.as_str().to_lowercase()
                    ),
                },
            )],
        );

        // The node containing the icon and title
        let title_node = kuchikiki::NodeRef::new_element(
            QualName::new(None, ns!(html), local_name!("p")),
            [(
                kuchikiki::ExpandedName::new(ns!(), local_name!("class")),
                kuchikiki::Attribute {
                    prefix: None,
                    value: "markdown-alert-title".to_string(),
                },
            )],
        );

        // The fist p element after the title
        let content_node =
            kuchikiki::NodeRef::new_element(QualName::new(None, ns!(html), local_name!("p")), None);

        content_node.append(kuchikiki::NodeRef::new_text(content_after_title));

        for child in first_p_children {
            content_node.append(child);
        }

        let mut title = alert_type.to_lowercase();
        title[..1].make_ascii_uppercase();

        title_node.append(kuchikiki::parse_html().one(*SVGS.get(&*title).expect(
            "We know this will never fail, since the regex will only match valid alert types",
        )));

        title_node.append(kuchikiki::NodeRef::new_text(title));

        alert_container_node.append(title_node);
        alert_container_node.append(content_node);

        for child in blockquote_children {
            alert_container_node.append(child);
        }

        // Replace the blockquote with the alert container
        blockquote.insert_after(alert_container_node);
        blockquote.detach();
    }

    // Serialize the modified DOM back to HTML
    let mut output = Vec::new();
    document
        .serialize(&mut output)
        .expect("Serialization should never fail, if it does there is a bug.");
    String::from_utf8(output).expect("Converting document should never fail.")
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
    use kuchikiki::{traits::*, ElementData, NodeDataRef};

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

        let mut elements = html_ouput.into_iter().map(|h| {
            let document = kuchikiki::parse_html().one(h);

            document
                .select_first(r#"a"#)
                .expect("Selector is hard-coded.")
        });

        assert_link(elements.next().unwrap(), false);
        assert_link(elements.next().unwrap(), false);
        assert_link(elements.next().unwrap(), true);
        assert_link(elements.next().unwrap(), true);
        assert_link(elements.next().unwrap(), true);

        // make sure the iterator is empty
        assert!(elements.next().is_none());
    }

    fn assert_link(element: NodeDataRef<ElementData>, internal: bool) {
        let attributes = element.attributes.borrow();
        if internal {
            let _onclick = Some(&format!(
                "handle_redirect(\"{}\")",
                attributes.get("href").unwrap()
            ));

            assert!(matches!(attributes.get("onclick"), _onclick))
        } else {
            assert!(attributes.get("onclick").is_none())
        }
    }

    #[test]
    fn alerts() {
        let md_input = [
            r#"
> [!Warning]
> This should work
            "#,
            r#"
> [!Warning] This
> shouldn't work
            "#,
            r#"
> [!Note] 
> `should work`
            "#,
        ];

        let html_ouput = md_input.map(md_to_html);

        let mut elements = html_ouput.into_iter().map(|h| {
            let document = kuchikiki::parse_html().one(h);

            document.select_first(r#"div"#)
        });

        assert!(elements.next().unwrap().is_ok_and(is_alert));
        assert!(elements.next().unwrap().is_err());
        assert!(elements.next().unwrap().is_ok_and(is_alert));

        // make sure the iterator is empty
        assert!(elements.next().is_none());
    }

    fn is_alert(element: NodeDataRef<ElementData>) -> bool {
        let attributes = element.attributes.borrow();
        attributes
            .get("class")
            .is_some_and(|c| c.contains("markdown-alert"))
            && element.as_node().select_first("p").is_ok_and(|p| {
                p.attributes
                    .borrow()
                    .get("class")
                    .is_some_and(|c| c.contains("markdown-alert-title"))
            })
    }
}
