//! Language definitions for LOC Counter
//!
//! This module contains language detection and comment syntax definitions.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;

/// Comment syntax for a language
#[derive(Debug, Clone)]
pub struct CommentSyntax {
    /// Single-line comment prefixes (e.g., "//", "#")
    pub line_comments: Vec<&'static str>,
    /// Block comment start/end pairs (e.g., ("/*", "*/"))
    pub block_comments: Vec<(&'static str, &'static str)>,
}

impl Default for CommentSyntax {
    fn default() -> Self {
        Self {
            line_comments: vec![],
            block_comments: vec![],
        }
    }
}

/// Language definition
#[derive(Debug, Clone)]
pub struct Language {
    /// Language name
    pub name: &'static str,
    /// File extensions (without dot)
    pub extensions: Vec<&'static str>,
    /// Comment syntax
    pub comments: CommentSyntax,
}

/// Get all supported languages
fn get_languages() -> Vec<Language> {
    vec![
        Language {
            name: "Rust",
            extensions: vec!["rs"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "TypeScript",
            extensions: vec!["ts", "tsx"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "JavaScript",
            extensions: vec!["js", "jsx", "mjs", "cjs"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "Python",
            extensions: vec!["py", "pyw", "pyi"],
            comments: CommentSyntax {
                line_comments: vec!["#"],
                block_comments: vec![("\"\"\"", "\"\"\""), ("'''", "'''")],
            },
        },
        Language {
            name: "Go",
            extensions: vec!["go"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "Java",
            extensions: vec!["java"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "C",
            extensions: vec!["c", "h"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "C++",
            extensions: vec!["cpp", "hpp", "cc", "cxx", "hxx"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "C#",
            extensions: vec!["cs"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "HTML",
            extensions: vec!["html", "htm"],
            comments: CommentSyntax {
                line_comments: vec![],
                block_comments: vec![("<!--", "-->")],
            },
        },
        Language {
            name: "CSS",
            extensions: vec!["css"],
            comments: CommentSyntax {
                line_comments: vec![],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "SCSS",
            extensions: vec!["scss", "sass"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "JSON",
            extensions: vec!["json"],
            comments: CommentSyntax::default(), // JSON has no comments
        },
        Language {
            name: "YAML",
            extensions: vec!["yaml", "yml"],
            comments: CommentSyntax {
                line_comments: vec!["#"],
                block_comments: vec![],
            },
        },
        Language {
            name: "TOML",
            extensions: vec!["toml"],
            comments: CommentSyntax {
                line_comments: vec!["#"],
                block_comments: vec![],
            },
        },
        Language {
            name: "Markdown",
            extensions: vec!["md", "markdown"],
            comments: CommentSyntax::default(), // Markdown doesn't have traditional comments
        },
        Language {
            name: "Shell",
            extensions: vec!["sh", "bash", "zsh"],
            comments: CommentSyntax {
                line_comments: vec!["#"],
                block_comments: vec![],
            },
        },
        Language {
            name: "Ruby",
            extensions: vec!["rb"],
            comments: CommentSyntax {
                line_comments: vec!["#"],
                block_comments: vec![("=begin", "=end")],
            },
        },
        Language {
            name: "PHP",
            extensions: vec!["php"],
            comments: CommentSyntax {
                line_comments: vec!["//", "#"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "Swift",
            extensions: vec!["swift"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "Kotlin",
            extensions: vec!["kt", "kts"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "SQL",
            extensions: vec!["sql"],
            comments: CommentSyntax {
                line_comments: vec!["--"],
                block_comments: vec![("/*", "*/")],
            },
        },
        Language {
            name: "Vue",
            extensions: vec!["vue"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("<!--", "-->"), ("/*", "*/")],
            },
        },
        Language {
            name: "Svelte",
            extensions: vec!["svelte"],
            comments: CommentSyntax {
                line_comments: vec!["//"],
                block_comments: vec![("<!--", "-->"), ("/*", "*/")],
            },
        },
        Language {
            name: "XML",
            extensions: vec!["xml", "xsl", "xsd"],
            comments: CommentSyntax {
                line_comments: vec![],
                block_comments: vec![("<!--", "-->")],
            },
        },
    ]
}

// Static cached languages list
static LANGUAGES: Lazy<Vec<Language>> = Lazy::new(get_languages);

// Extension to language name mapping for O(1) lookup
static EXT_TO_LANG: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for lang in &*LANGUAGES {
        for ext in &lang.extensions {
            map.insert(ext.to_string(), lang.name);
        }
    }
    map
});

// Language name to comment syntax mapping for O(1) lookup
static LANG_TO_SYNTAX: Lazy<HashMap<&'static str, CommentSyntax>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for lang in &*LANGUAGES {
        map.insert(lang.name, lang.comments.clone());
    }
    map
});

/// Detect language from file path
pub fn detect_language(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    EXT_TO_LANG.get(&extension).copied()
}

/// Get comment syntax for a language
pub fn get_comment_syntax(language_name: &str) -> CommentSyntax {
    LANG_TO_SYNTAX
        .get(language_name)
        .cloned()
        .unwrap_or_default()
}

/// Check if a file extension is supported
pub fn is_supported_extension(extension: &str) -> bool {
    EXT_TO_LANG.contains_key(&extension.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_language_rust() {
        let path = PathBuf::from("src/main.rs");
        assert_eq!(detect_language(&path), Some("Rust"));
    }

    #[test]
    fn test_detect_language_typescript() {
        let path = PathBuf::from("src/app.ts");
        assert_eq!(detect_language(&path), Some("TypeScript"));

        let path = PathBuf::from("src/App.tsx");
        assert_eq!(detect_language(&path), Some("TypeScript"));
    }

    #[test]
    fn test_detect_language_javascript() {
        let path = PathBuf::from("src/index.js");
        assert_eq!(detect_language(&path), Some("JavaScript"));

        let path = PathBuf::from("src/App.jsx");
        assert_eq!(detect_language(&path), Some("JavaScript"));
    }

    #[test]
    fn test_detect_language_python() {
        let path = PathBuf::from("script.py");
        assert_eq!(detect_language(&path), Some("Python"));
    }

    #[test]
    fn test_detect_language_go() {
        let path = PathBuf::from("main.go");
        assert_eq!(detect_language(&path), Some("Go"));
    }

    #[test]
    fn test_detect_language_unknown() {
        let path = PathBuf::from("file.unknown");
        assert_eq!(detect_language(&path), None);
    }

    #[test]
    fn test_detect_language_no_extension() {
        let path = PathBuf::from("Makefile");
        assert_eq!(detect_language(&path), None);
    }

    #[test]
    fn test_detect_language_case_insensitive() {
        let path = PathBuf::from("file.RS");
        assert_eq!(detect_language(&path), Some("Rust"));

        let path = PathBuf::from("file.Py");
        assert_eq!(detect_language(&path), Some("Python"));
    }

    #[test]
    fn test_get_comment_syntax_rust() {
        let syntax = get_comment_syntax("Rust");
        assert!(syntax.line_comments.contains(&"//"));
        assert!(syntax.block_comments.iter().any(|(s, e)| *s == "/*" && *e == "*/"));
    }

    #[test]
    fn test_get_comment_syntax_python() {
        let syntax = get_comment_syntax("Python");
        assert!(syntax.line_comments.contains(&"#"));
        assert!(syntax.block_comments.iter().any(|(s, e)| *s == "\"\"\"" && *e == "\"\"\""));
    }

    #[test]
    fn test_get_comment_syntax_html() {
        let syntax = get_comment_syntax("HTML");
        assert!(syntax.line_comments.is_empty());
        assert!(syntax.block_comments.iter().any(|(s, e)| *s == "<!--" && *e == "-->"));
    }

    #[test]
    fn test_get_comment_syntax_json() {
        let syntax = get_comment_syntax("JSON");
        assert!(syntax.line_comments.is_empty());
        assert!(syntax.block_comments.is_empty());
    }

    #[test]
    fn test_get_comment_syntax_unknown() {
        let syntax = get_comment_syntax("Unknown");
        assert!(syntax.line_comments.is_empty());
        assert!(syntax.block_comments.is_empty());
    }

    #[test]
    fn test_is_supported_extension() {
        assert!(is_supported_extension("rs"));
        assert!(is_supported_extension("ts"));
        assert!(is_supported_extension("tsx"));
        assert!(is_supported_extension("py"));
        assert!(is_supported_extension("go"));
        assert!(is_supported_extension("json"));
        assert!(is_supported_extension("md"));
        assert!(!is_supported_extension("unknown"));
        assert!(!is_supported_extension("exe"));
    }

    #[test]
    fn test_is_supported_extension_case_insensitive() {
        assert!(is_supported_extension("RS"));
        assert!(is_supported_extension("Py"));
        assert!(is_supported_extension("JSON"));
    }

    #[test]
    fn test_languages_count() {
        // At least 20 languages should be defined
        assert!(LANGUAGES.len() >= 20);
    }

    #[test]
    fn test_all_languages_have_names() {
        for lang in &*LANGUAGES {
            assert!(!lang.name.is_empty());
            assert!(!lang.extensions.is_empty());
        }
    }
}
