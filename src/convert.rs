use kuchikiki::traits::*;
use markdown::{to_html_with_options, Options};

pub fn convert(md: &str) -> String {
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

    // Find elements matching the selector
    let matching_elements = document
        .select("li>p>input[type=\"checkbox\"]")
        .expect("The selector is hard-coded.");

    for element in matching_elements {
        let checkbox = element.as_node();
        let li = checkbox
            .parent()
            .expect("The selector determines that these exist")
            .parent()
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

    // Serialize the modified DOM back to HTML
    let mut output = Vec::new();
    document
        .serialize(&mut output)
        .expect("Serialization should never fail. All we did was add some classes.");
    String::from_utf8(output).expect("Converting to valid output should never fail.")
}
