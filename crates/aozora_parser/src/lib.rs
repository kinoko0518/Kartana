//! # aozora_parser
//!
//! A library for parsing Aozora Bunko format texts and generating EPUB files.
//!
//! ## Quick Start (Simple)
//!
//! ```ignore
//! use aozora_parser::{text_to_xhtml, text_to_epub};
//!
//! // Convert directly to XHTML
//! let (xhtml, toc) = text_to_xhtml(text)?;
//!
//! // Or generate EPUB file directly
//! text_to_epub(text, "output.epub")?;
//! ```
//!
//! ## Advanced Usage
//!
//! For more control over the conversion process:
//!
//! ```ignore
//! use aozora_parser::{parse_aozora, parse, parse_blocks, EpubGenerator};
//!
//! let tokens = parse_aozora(text)?;
//! let doc = parse(tokens)?;
//! let blocks = parse_blocks(doc.items)?;
//! let generator = EpubGenerator::new(doc.metadata.title, doc.metadata.author, blocks);
//! generator.write_to_file("output.epub")?;
//! ```

use std::path::Path;

// Internal modules (implementation details)
mod tokenizer;
mod parser;
mod block_parser;
mod linter;
mod xhtml_generator;
mod epub_generator;
mod css;

// Re-export main entry point functions
pub use tokenizer::parse_aozora;
pub use parser::parse;
pub use block_parser::parse_blocks;
pub use linter::lint;
pub use css::default_css;

// Re-export primary types for working with documents
pub use parser::{AozoraDocument, AozoraMetadata, ParsedItem, DecoratedText, SpecialCharacter, ParseError};
pub use block_parser::{AozoraBlock, BlockElement, BlockParseError};
pub use tokenizer::{AozoraToken, Span, TokenizeError};
pub use linter::{LintResult, LintWarning, LintWarningKind, Severity};

// Re-export generators
pub use epub_generator::EpubGenerator;
pub use xhtml_generator::{XhtmlGenerator, TocEntry};

// Re-export command types for advanced usage (matching decorations, etc.)
pub mod command {
    //! Command types used in Aozora Bunko formatting.
    pub use crate::tokenizer::command::*;
}

/// Result of XHTML conversion.
#[derive(Debug, Clone)]
pub struct XhtmlOutput {
    /// Generated XHTML string
    pub xhtml: String,
    /// Table of contents entries
    pub toc: Vec<TocEntry>,
    /// Document metadata (title, author)
    pub metadata: AozoraMetadata,
}

/// Result of XHTML conversion with lint warnings.
#[derive(Debug, Clone)]
pub struct XhtmlOutputWithLint {
    /// Generated XHTML string
    pub xhtml: String,
    /// Table of contents entries
    pub toc: Vec<TocEntry>,
    /// Document metadata (title, author)
    pub metadata: AozoraMetadata,
    /// Lint warnings
    pub warnings: Vec<LintWarning>,
}

/// Error type for high-level conversion functions.
#[derive(Debug)]
pub enum ConversionError {
    /// Error during tokenization
    Tokenize(TokenizeError),
    /// Error during parsing
    Parse(ParseError),
    /// Error during block parsing
    BlockParse(BlockParseError),
    /// Error during file I/O
    Io(std::io::Error),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::Tokenize(e) => write!(f, "Tokenization error: {:?}", e),
            ConversionError::Parse(e) => write!(f, "Parse error: {:?}", e),
            ConversionError::BlockParse(e) => write!(f, "Block parse error: {:?}", e),
            ConversionError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<TokenizeError> for ConversionError {
    fn from(e: TokenizeError) -> Self { ConversionError::Tokenize(e) }
}

impl From<ParseError> for ConversionError {
    fn from(e: ParseError) -> Self { ConversionError::Parse(e) }
}

impl From<BlockParseError> for ConversionError {
    fn from(e: BlockParseError) -> Self { ConversionError::BlockParse(e) }
}

impl From<std::io::Error> for ConversionError {
    fn from(e: std::io::Error) -> Self { ConversionError::Io(e) }
}

/// Converts Aozora Bunko format text directly to XHTML.
///
/// This is a high-level convenience function that handles the entire conversion
/// pipeline internally: tokenization → parsing → block parsing → XHTML generation.
///
/// # Arguments
///
/// * `text` - The Aozora Bunko format text to convert
///
/// # Example
///
/// ```ignore
/// let output = aozora_parser::text_to_xhtml(aozora_text)?;
/// println!("Title: {}", output.metadata.title);
/// println!("TOC entries: {}", output.toc.len());
/// ```
pub fn text_to_xhtml(text: String) -> Result<XhtmlOutput, ConversionError> {
    let tokens = parse_aozora(text)?;
    let doc = parse(tokens)?;
    let blocks = parse_blocks(doc.items)?;
    let (xhtml, toc) = XhtmlGenerator::generate(&blocks, &doc.metadata.title);
    Ok(XhtmlOutput {
        xhtml,
        toc,
        metadata: doc.metadata,
    })
}

/// Converts Aozora Bunko format text directly to an EPUB file.
///
/// This is a high-level convenience function that handles the entire conversion
/// pipeline internally and writes the result to the specified path.
///
/// # Arguments
///
/// * `text` - The Aozora Bunko format text to convert
/// * `path` - The output file path for the EPUB
///
/// # Example
///
/// ```ignore
/// aozora_parser::text_to_epub(aozora_text, "output.epub")?;
/// ```
pub fn text_to_epub<P: AsRef<Path>>(text: String, path: P) -> Result<(), ConversionError> {
    let tokens = parse_aozora(text)?;
    let doc = parse(tokens)?;
    let blocks = parse_blocks(doc.items)?;
    let generator = EpubGenerator::new(doc.metadata.title, doc.metadata.author, blocks);
    generator.write_to_file(path)?;
    Ok(())
}

/// Converts Aozora Bunko format text to XHTML with lint warnings.
///
/// This function runs the linter and returns any warnings found along with
/// the generated XHTML.
///
/// # Arguments
///
/// * `text` - The Aozora Bunko format text to convert
///
/// # Example
///
/// ```ignore
/// let output = aozora_parser::text_to_xhtml_with_lint(aozora_text)?;
/// for w in &output.warnings {
///     println!("[{:?}] {}", w.severity, w.message);
/// }
/// ```
pub fn text_to_xhtml_with_lint(text: String) -> Result<XhtmlOutputWithLint, ConversionError> {
    let original = text.clone();
    let tokens = parse_aozora(text)?;
    let doc = parse(tokens)?;
    let blocks = parse_blocks(doc.items)?;
    
    // Run linter
    let lint_result = lint(blocks, &original);
    
    let (xhtml, toc) = XhtmlGenerator::generate(&lint_result.block, &doc.metadata.title);
    Ok(XhtmlOutputWithLint {
        xhtml,
        toc,
        metadata: doc.metadata,
        warnings: lint_result.warnings,
    })
}
