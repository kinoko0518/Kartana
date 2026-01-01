//! Linter module for validating Aozora Bunko text formatting.
//!
//! This module provides lint warnings for common formatting issues
//! without stopping the parsing process.

use crate::block_parser::{AozoraBlock, BlockElement};
use crate::parser::{DecoratedText, ParsedItem};
use crate::tokenizer::Span;

/// Severity level of a lint warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error: Should be fixed
    Error,
    /// Warning: Recommended to fix
    Warning,
    /// Info: Informational only
    Info,
}

/// Kind of lint warning.
#[derive(Debug, Clone, PartialEq)]
pub enum LintWarningKind {
    // === 構文関連 ===
    /// ルビが対応するテキストなしで出現
    RubyWithoutText,
    /// 未知のコマンド
    UnknownCommand(String),
    /// 開始タグと終了タグの不一致
    MismatchedBlockTags,

    // === 表記関連 ===
    /// 段落先頭に字下げがない
    MissingParagraphIndent,
    /// 。」または．」パターン
    PunctuationBeforeQuote,
    /// …または―が奇数個連続
    OddEllipsisCount,
    /// ！？の後に不正な文字
    InvalidCharAfterExclamation,
}

/// A lint warning with location and message.
#[derive(Debug, Clone)]
pub struct LintWarning {
    /// Kind of warning
    pub kind: LintWarningKind,
    /// Location in original text
    pub span: Span,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
}

impl LintWarning {
    /// Create a new warning.
    pub fn new(kind: LintWarningKind, span: Span, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            severity,
            message: message.into(),
        }
    }

    /// Create an error-level warning.
    pub fn error(kind: LintWarningKind, span: Span, message: impl Into<String>) -> Self {
        Self::new(kind, span, Severity::Error, message)
    }

    /// Create a warning-level warning.
    pub fn warning(kind: LintWarningKind, span: Span, message: impl Into<String>) -> Self {
        Self::new(kind, span, Severity::Warning, message)
    }

    /// Create an info-level warning.
    pub fn info(kind: LintWarningKind, span: Span, message: impl Into<String>) -> Self {
        Self::new(kind, span, Severity::Info, message)
    }
}

/// Result of linting.
#[derive(Debug, Clone)]
pub struct LintResult {
    /// The block (unchanged)
    pub block: AozoraBlock,
    /// Collected warnings
    pub warnings: Vec<LintWarning>,
}

/// Lint an AozoraBlock and return warnings.
///
/// This function validates the block against common formatting rules
/// and returns any warnings found.
///
/// # Arguments
///
/// * `block` - The parsed block to lint
/// * `original_text` - The original text (used for text-level checks)
///
/// # Example
///
/// ```ignore
/// let result = lint(block, &original_text);
/// for warning in &result.warnings {
///     println!("{}: {}", warning.severity, warning.message);
/// }
/// ```
pub fn lint(block: AozoraBlock, original_text: &str) -> LintResult {
    let mut warnings = Vec::new();
    
    // Run all lint checks
    check_paragraph_indent(&block, &mut warnings);
    check_text_patterns(original_text, &mut warnings);
    
    LintResult { block, warnings }
}

/// Check for proper paragraph indentation.
fn check_paragraph_indent(block: &AozoraBlock, warnings: &mut Vec<LintWarning>) {
    let mut after_newline = true; // Start of document counts as after newline
    
    for elem in &block.elements {
        match elem {
            BlockElement::Item(item) => {
                match item {
                    ParsedItem::Newline(_) => {
                        after_newline = true;
                    }
                    ParsedItem::Text(dt) if after_newline => {
                        // Check if paragraph starts with proper indent
                        if !is_valid_paragraph_start(&dt.text) {
                            warnings.push(LintWarning::warning(
                                LintWarningKind::MissingParagraphIndent,
                                dt.span,
                                "段落の先頭には全角スペースまたは字下げが必要です",
                            ));
                        }
                        after_newline = false;
                    }
                    ParsedItem::Command { .. } => {
                        // Commands like 字下げ are valid paragraph starts
                        after_newline = false;
                    }
                    _ => {
                        after_newline = false;
                    }
                }
            }
            BlockElement::Block(sub_block) => {
                // Recursively check nested blocks
                check_paragraph_indent(sub_block, warnings);
                after_newline = false;
            }
        }
    }
}

/// Check if a paragraph starts with valid indentation.
fn is_valid_paragraph_start(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    
    let first_char = text.chars().next().unwrap();
    
    // Valid starts:
    // - Full-width space (indent)
    // - Opening quote (dialogue)
    // - Various brackets
    matches!(first_char, 
        '　' |  // full-width space
        '「' | '『' | '（' | '【' | '〈' | '《' |  // opening brackets
        '─' | '―' | '…'  // decorative starts
    )
}

/// Check text patterns for common issues.
fn check_text_patterns(text: &str, warnings: &mut Vec<LintWarning>) {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0;
    
    while pos < chars.len() {
        let c = chars[pos];
        
        // Check 。」 or ．」 pattern
        if (c == '。' || c == '．') && pos + 1 < chars.len() && chars[pos + 1] == '」' {
            warnings.push(LintWarning::warning(
                LintWarningKind::PunctuationBeforeQuote,
                Span::new(pos, pos + 2),
                "閉じ括弧は句点と同じ効果を持つため、句点との併用は冗長です",
            ));
        }
        
        // Check odd ellipsis/dash count
        if c == '…' || c == '―' {
            let start = pos;
            let target = c;
            let mut count = 0;
            while pos < chars.len() && chars[pos] == target {
                count += 1;
                pos += 1;
            }
            if count % 2 != 0 {
                let char_name = if target == '…' { "三点リーダ" } else { "ダッシュ" };
                warnings.push(LintWarning::warning(
                    LintWarningKind::OddEllipsisCount,
                    Span::new(start, pos),
                    format!("{}は偶数個（2個）で使用することが推奨されます", char_name),
                ));
            }
            continue; // Already advanced pos
        }
        
        // Check character after ！ or ？
        if (c == '！' || c == '？') && pos + 1 < chars.len() {
            let next = chars[pos + 1];
            if !is_valid_after_exclamation(next) {
                warnings.push(LintWarning::warning(
                    LintWarningKind::InvalidCharAfterExclamation,
                    Span::new(pos, pos + 2),
                    "！？の後には空白または閉じ括弧が必要です",
                ));
            }
        }
        
        pos += 1;
    }
}

/// Check if a character is valid after ！ or ？
fn is_valid_after_exclamation(c: char) -> bool {
    matches!(c,
        '！' | '？' |  // Another exclamation/question
        '」' | '』' | '）' | '】' | '〉' | '》' |  // Closing brackets
        '　' | ' ' | '\n' | '\r'  // Whitespace
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_parser::parse_blocks;
    use crate::parser::parse;
    use crate::tokenizer::parse_aozora;

    #[test]
    fn test_punctuation_before_quote() {
        let text = "タイトル\n著者\nこれは文章です。」と言った。";
        let mut warnings = Vec::new();
        check_text_patterns(text, &mut warnings);
        
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, LintWarningKind::PunctuationBeforeQuote));
    }

    #[test]
    fn test_odd_ellipsis() {
        let text = "タイトル\n著者\nこれは…途中";
        let mut warnings = Vec::new();
        check_text_patterns(text, &mut warnings);
        
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, LintWarningKind::OddEllipsisCount));
    }

    #[test]
    fn test_even_ellipsis_ok() {
        let text = "これは……途中";
        let mut warnings = Vec::new();
        check_text_patterns(text, &mut warnings);
        
        // No warnings for even count
        let ellipsis_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.kind, LintWarningKind::OddEllipsisCount))
            .collect();
        assert!(ellipsis_warnings.is_empty());
    }

    #[test]
    fn test_invalid_char_after_exclamation() {
        let text = "びっくり！だね";
        let mut warnings = Vec::new();
        check_text_patterns(text, &mut warnings);
        
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0].kind, LintWarningKind::InvalidCharAfterExclamation));
    }

    #[test]
    fn test_valid_after_exclamation() {
        let text = "びっくり！　続き";
        let mut warnings = Vec::new();
        check_text_patterns(text, &mut warnings);
        
        let excl_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.kind, LintWarningKind::InvalidCharAfterExclamation))
            .collect();
        assert!(excl_warnings.is_empty());
    }
}
