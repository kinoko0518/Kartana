//! Decoration system for the Kartana Editor.
//!
//! This module provides VSCode-like text decorations for syntax highlighting
//! and lint warnings.

use aozora_parser::linter::{LintWarning, Severity};
use aozora_parser::tokenizer::AozoraToken;

/// Types of decorations for Aozora text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationType {
    /// Command: ［＃...］
    Command,
    /// Ruby text: 《...》
    Ruby,
    /// Ruby separator: ｜
    RubySeparator,
    /// Kanji (base text for ruby)
    Kanji,
    /// Hiragana
    Hiragana,
    /// Katakana
    Katakana,
    /// Lint error
    LintError,
    /// Lint warning
    LintWarning,
    /// Lint info
    LintInfo,
}

impl DecorationType {
    /// Get the CSS class for this decoration type.
    pub fn css_class(&self) -> &'static str {
        match self {
            DecorationType::Command => "deco-command",
            DecorationType::Ruby => "deco-ruby",
            DecorationType::RubySeparator => "deco-ruby-sep",
            DecorationType::Kanji => "deco-kanji",
            DecorationType::Hiragana => "deco-hiragana",
            DecorationType::Katakana => "deco-katakana",
            DecorationType::LintError => "deco-lint-error",
            DecorationType::LintWarning => "deco-lint-warning",
            DecorationType::LintInfo => "deco-lint-info",
        }
    }
}

/// A decoration applied to a text range.
#[derive(Debug, Clone, PartialEq)]
pub struct Decoration {
    /// Start position (character index, 0-based)
    pub start: usize,
    /// End position (character index, exclusive)
    pub end: usize,
    /// Type of decoration
    pub decoration_type: DecorationType,
    /// Optional hover message (for lint warnings)
    pub hover_message: Option<String>,
}

impl Decoration {
    /// Create a new decoration.
    pub fn new(start: usize, end: usize, decoration_type: DecorationType) -> Self {
        Self {
            start,
            end,
            decoration_type,
            hover_message: None,
        }
    }

    /// Create a new decoration with a hover message.
    pub fn with_message(
        start: usize,
        end: usize,
        decoration_type: DecorationType,
        message: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end,
            decoration_type,
            hover_message: Some(message.into()),
        }
    }
}

/// Compute decorations from tokenizer output.
pub fn decorations_from_tokens(tokens: &[AozoraToken]) -> Vec<Decoration> {
    let mut decorations = Vec::new();

    for token in tokens {
        match token {
            AozoraToken::Command(cmd) => {
                decorations.push(Decoration::new(
                    cmd.span.start,
                    cmd.span.end,
                    DecorationType::Command,
                ));
            }
            AozoraToken::Ruby { span, .. } => {
                decorations.push(Decoration::new(span.start, span.end, DecorationType::Ruby));
            }
            AozoraToken::RubySeparator(span) => {
                decorations.push(Decoration::new(
                    span.start,
                    span.end,
                    DecorationType::RubySeparator,
                ));
            }
            AozoraToken::Text(text_token) => {
                use aozora_parser::tokenizer::TextKind;
                let deco_type = match text_token.kind {
                    TextKind::Kanji => DecorationType::Kanji,
                    TextKind::Hiragana => DecorationType::Hiragana,
                    TextKind::Katakana => DecorationType::Katakana,
                    TextKind::Other => continue, // Don't decorate "other" text
                };
                decorations.push(Decoration::new(
                    text_token.span.start,
                    text_token.span.end,
                    deco_type,
                ));
            }
            _ => {}
        }
    }

    decorations
}

/// Compute decorations from lint warnings.
pub fn decorations_from_lint(warnings: &[LintWarning]) -> Vec<Decoration> {
    warnings
        .iter()
        .map(|warning| {
            let deco_type = match warning.severity {
                Severity::Error => DecorationType::LintError,
                Severity::Warning => DecorationType::LintWarning,
                Severity::Info => DecorationType::LintInfo,
            };
            Decoration::with_message(
                warning.span.start,
                warning.span.end,
                deco_type,
                &warning.message,
            )
        })
        .collect()
}

/// Merge and sort decorations, handling overlaps.
/// Later decorations in the input take precedence for overlapping regions.
pub fn merge_decorations(mut decorations: Vec<Decoration>) -> Vec<Decoration> {
    // Sort by start position, then by end position (longer ranges first)
    decorations.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.end.cmp(&a.end))
    });
    decorations
}

/// A segment of text with its decorations.
#[derive(Debug, Clone)]
pub struct DecoratedSegment {
    /// The text content
    pub text: String,
    /// CSS classes to apply
    pub classes: Vec<String>,
    /// Hover message (if any)
    pub hover_message: Option<String>,
}

/// Split text into decorated segments.
///
/// This function takes the original text and a list of decorations,
/// and produces segments that can be rendered as HTML spans.
pub fn split_into_segments(text: &str, decorations: &[Decoration]) -> Vec<DecoratedSegment> {
    if text.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = text.chars().collect();
    let mut segments = Vec::new();

    // Build a map of position -> active decorations
    let mut boundaries: Vec<usize> = vec![0, chars.len()];
    for deco in decorations {
        if deco.start < chars.len() {
            boundaries.push(deco.start);
        }
        if deco.end <= chars.len() {
            boundaries.push(deco.end);
        }
    }
    boundaries.sort();
    boundaries.dedup();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];

        if start >= end || start >= chars.len() {
            continue;
        }

        let segment_text: String = chars[start..end.min(chars.len())].iter().collect();

        // Find all decorations that cover this segment
        let mut classes = Vec::new();
        let mut hover_message = None;

        for deco in decorations {
            if deco.start <= start && deco.end >= end {
                classes.push(deco.decoration_type.css_class().to_string());
                if deco.hover_message.is_some() && hover_message.is_none() {
                    hover_message = deco.hover_message.clone();
                }
            }
        }

        segments.push(DecoratedSegment {
            text: segment_text,
            classes,
            hover_message,
        });
    }

    segments
}

/// Render decorated segments to HTML string.
pub fn render_to_html(segments: &[DecoratedSegment]) -> String {
    let mut html = String::new();

    for segment in segments {
        // Escape HTML entities
        let escaped_text = segment
            .text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\r', "") // Remove CR to avoid confusion
            .replace('\n', "<br>");

        if segment.classes.is_empty() && segment.hover_message.is_none() {
            html.push_str(&escaped_text);
        } else {
            let classes = segment.classes.join(" ");
            let title_attr = segment
                .hover_message
                .as_ref()
                .map(|msg| format!(" title=\"{}\"", msg.replace('"', "&quot;")))
                .unwrap_or_default();

            html.push_str(&format!(
                "<span class=\"{}\"{}>{}</span>",
                classes, title_attr, escaped_text
            ));
        }
    }

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoration_css_class() {
        assert_eq!(DecorationType::Command.css_class(), "deco-command");
        assert_eq!(DecorationType::LintError.css_class(), "deco-lint-error");
    }

    #[test]
    fn test_split_into_segments() {
        let text = "漢字《かんじ》";
        let decorations = vec![
            Decoration::new(0, 2, DecorationType::Kanji),
            Decoration::new(2, 7, DecorationType::Ruby),
        ];

        let segments = split_into_segments(text, &decorations);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "漢字");
        assert!(segments[0].classes.contains(&"deco-kanji".to_string()));
        assert_eq!(segments[1].text, "《かんじ》");
        assert!(segments[1].classes.contains(&"deco-ruby".to_string()));
    }

    #[test]
    fn test_render_to_html() {
        let segments = vec![
            DecoratedSegment {
                text: "Hello".to_string(),
                classes: vec!["deco-command".to_string()],
                hover_message: None,
            },
            DecoratedSegment {
                text: " World".to_string(),
                classes: vec![],
                hover_message: None,
            },
        ];

        let html = render_to_html(&segments);
        assert_eq!(html, "<span class=\"deco-command\">Hello</span> World");
    }
}
