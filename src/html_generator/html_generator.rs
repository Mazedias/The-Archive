use std::fs;
use pulldown_cmark::{Parser, Options, html};

use crate::file_manager::file_structure::FileEntry;

/// Generates a string in HTML format from a markdown file. 
/// 
/// # Arguments
/// 
/// * `file` - path of the file starting from within the content folder (`content/*file*`)
/// 
/// # Returns
/// A string in HTML format if parse was succesful otherwise a string containing an error message.
pub fn markdown_to_html(file: &str) -> String {
    let path = format!("{}.md", file);
    let markdown = fs::read_to_string(path).unwrap_or_else(|_| "# 404\n\nFile not found.".to_string());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}


/// Tranforms a `FileEntry` instance into a HTML string
/// 
/// # Arguments
/// 
/// * `entry` - Refenrence to a `FileEntry`
/// 
/// # Returns
/// A string containing HTML formated text representing the (possible) nested structure of `entry`.
pub fn build_html_structure(entry: &FileEntry) -> String {
    match entry {
        FileEntry::File { name, path, .. } => {
            let normalized_path = path.replace("\\", "/");

            format!(
                r#"
                <div class="file-container">
                    <button onclick="loadContent('{}')" class="file-link">
                        <i class="fa-solid fa-file file-icon"></i>
                        {}
                    </button>
                </div> 
                "#,
                normalized_path,
                name
            )
        },
        FileEntry::Directory { name, children, .. } => {
            let child_html: Vec<String> = children.iter().map(build_html_structure).collect();

            format!(
                r#"
                <div class="dropdown-container">
                    <button class="dropdown-button">
                        {}
                    </button>
                    <div class="dropdown-content">
                        {}
                    </div>
                </div>
                "#,
                name,
                child_html.join("\n"),
            )
        }
    }
}
