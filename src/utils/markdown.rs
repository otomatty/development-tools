//! Markdown Rendering Utility
//!
//! Provides functions to render Markdown content to HTML
//! with GitHub Flavored Markdown support and XSS sanitization.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/issue_detail_modal.rs
//! Dependencies:
//!   ├─ pulldown-cmark (external crate)
//!   └─ ammonia (external crate)

use ammonia::Builder;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashSet;

/// Render Markdown content to sanitized HTML
///
/// Supports GitHub Flavored Markdown features:
/// - Tables
/// - Strikethrough
/// - Task lists
/// - Autolinks
///
/// Also sanitizes the output to prevent XSS attacks.
pub fn render_markdown(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }

    // Configure pulldown-cmark options for GFM support
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    // Parse and render to HTML
    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Sanitize the output (allow safe HTML tags commonly used in GitHub)
    sanitize_html(&html_output)
}

/// Sanitize HTML content to prevent XSS attacks
/// Allows a safe subset of HTML tags commonly used in GitHub Markdown
fn sanitize_html(html: &str) -> String {
    // Define allowed tags for GitHub-style markdown
    let mut tags = HashSet::new();
    for tag in &[
        "a",
        "abbr",
        "b",
        "blockquote",
        "br",
        "code",
        "dd",
        "del",
        "details",
        "div",
        "dl",
        "dt",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "img",
        "input",
        "ins",
        "kbd",
        "li",
        "ol",
        "p",
        "pre",
        "q",
        "s",
        "samp",
        "span",
        "strong",
        "sub",
        "summary",
        "sup",
        "table",
        "tbody",
        "td",
        "tfoot",
        "th",
        "thead",
        "tr",
        "ul",
        "var",
    ] {
        tags.insert(*tag);
    }

    // Define allowed attributes
    let mut tag_attributes = std::collections::HashMap::new();

    // Links (rel is set via link_rel, not here)
    let mut a_attrs = HashSet::new();
    a_attrs.insert("href");
    a_attrs.insert("title");
    tag_attributes.insert("a", a_attrs);

    // Images
    let mut img_attrs = HashSet::new();
    img_attrs.insert("src");
    img_attrs.insert("alt");
    img_attrs.insert("title");
    img_attrs.insert("width");
    img_attrs.insert("height");
    tag_attributes.insert("img", img_attrs);

    // Task list checkboxes
    let mut input_attrs = HashSet::new();
    input_attrs.insert("type");
    input_attrs.insert("checked");
    input_attrs.insert("disabled");
    tag_attributes.insert("input", input_attrs);

    // Table cells
    let mut td_attrs = HashSet::new();
    td_attrs.insert("align");
    td_attrs.insert("colspan");
    td_attrs.insert("rowspan");
    tag_attributes.insert("td", td_attrs.clone());
    tag_attributes.insert("th", td_attrs);

    // Code blocks
    let mut code_attrs = HashSet::new();
    code_attrs.insert("class");
    tag_attributes.insert("code", code_attrs.clone());
    tag_attributes.insert("pre", code_attrs.clone());
    tag_attributes.insert("div", code_attrs);

    Builder::default()
        .tags(tags)
        .tag_attributes(tag_attributes)
        .link_rel(Some("noopener noreferrer"))
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_input() {
        assert_eq!(render_markdown(""), "");
        assert_eq!(render_markdown("   "), "");
    }

    #[test]
    fn test_render_heading() {
        let result = render_markdown("# Hello World");
        assert!(result.contains("<h1>"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_render_bold_italic() {
        let result = render_markdown("**bold** and *italic*");
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_list() {
        let result = render_markdown("- item 1\n- item 2");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>"));
    }

    #[test]
    fn test_render_code_block() {
        let result = render_markdown("```rust\nfn main() {}\n```");
        assert!(result.contains("<pre>"));
        assert!(result.contains("<code"));
    }

    #[test]
    fn test_render_link() {
        let result = render_markdown("[GitHub](https://github.com)");
        assert!(result.contains("<a"));
        assert!(result.contains("href=\"https://github.com\""));
        assert!(result.contains("rel=\"noopener noreferrer\""));
    }

    #[test]
    fn test_render_table() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = render_markdown(input);
        assert!(result.contains("<table>"));
        assert!(result.contains("<th>"));
        assert!(result.contains("<td>"));
    }

    #[test]
    fn test_render_task_list() {
        let input = "- [x] Done\n- [ ] Todo";
        let result = render_markdown(input);
        assert!(result.contains("<input"));
        assert!(result.contains("checked"));
    }

    #[test]
    fn test_sanitize_script_tag() {
        let input = "<script>alert('xss')</script>";
        let result = render_markdown(input);
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_sanitize_onclick() {
        let input = "<a onclick=\"alert('xss')\" href=\"#\">link</a>";
        let result = render_markdown(input);
        assert!(!result.contains("onclick"));
    }
}
