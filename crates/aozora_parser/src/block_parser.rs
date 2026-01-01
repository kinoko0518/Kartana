use crate::parser::ParsedItem;
use crate::tokenizer::command::{Command, CommandBegin, CommandEnd};
use crate::tokenizer::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum BlockElement {
    Item(ParsedItem),
    Block(AozoraBlock),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AozoraBlock {
    pub decoration: Option<CommandBegin>, // None for Root
    pub elements: Vec<BlockElement>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockParseError {
    UnexpectedEnd { end: CommandEnd, span: Span },
    UnclosedBlock { begin: CommandBegin, span: Span },
}

/// Helper to get span from ParsedItem
fn item_span(item: &ParsedItem) -> Span {
    match item {
        ParsedItem::Text(dt) => dt.span,
        ParsedItem::Command { span, .. } => *span,
        ParsedItem::Newline(span) => *span,
        ParsedItem::SpecialCharacter { span, .. } => *span,
    }
}

/// Helper to get span from BlockElement
fn element_span(elem: &BlockElement) -> Span {
    match elem {
        BlockElement::Item(item) => item_span(item),
        BlockElement::Block(block) => block.span,
    }
}

pub fn parse_blocks(items: Vec<ParsedItem>) -> Result<AozoraBlock, BlockParseError> {
    let mut stack: Vec<AozoraBlock> = Vec::new();
    // Root block
    stack.push(AozoraBlock {
        decoration: None,
        elements: Vec::new(),
        span: Span::default(),
    });

    for item in items {
        if let ParsedItem::Command { cmd: Command::CommandBegin(begin), span } = &item {
            // Start a new block
            let new_block = AozoraBlock {
                decoration: Some(begin.clone()),
                elements: Vec::new(),
                span: *span,
            };
            stack.push(new_block);
        } else if let ParsedItem::Command { cmd: Command::CommandEnd(end), span } = &item {
            // Close the current block
            if stack.len() <= 1 {
                // Trying to close root or extra closing tag
                return Err(BlockParseError::UnexpectedEnd { end: end.clone(), span: *span });
            }

            let mut finished_block = stack.pop().unwrap();
            
            // Update span to include end command
            finished_block.span = finished_block.span.merge(span);
            
            // Add to parent
            if let Some(parent) = stack.last_mut() {
                parent.elements.push(BlockElement::Block(finished_block));
            }
        } else {
            // Standard item
            if let Some(current_block) = stack.last_mut() {
                current_block.elements.push(BlockElement::Item(item));
            }
        }
    }

    // Auto-close any unclosed blocks (some Aozora documents don't explicitly close all blocks)
    while stack.len() > 1 {
        let finished_block = stack.pop().unwrap();
        if let Some(parent) = stack.last_mut() {
            parent.elements.push(BlockElement::Block(finished_block));
        }
    }

    // Calculate root span from elements
    let mut root = stack.pop().unwrap();
    if !root.elements.is_empty() {
        let first_span = element_span(&root.elements[0]);
        let last_span = element_span(root.elements.last().unwrap());
        root.span = first_span.merge(&last_span);
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DecoratedText, SpecialCharacter};
    use crate::tokenizer::command::{
        Alignment, Midashi, MidashiSize, MidashiType, SingleCommand,
    };
    use crate::tokenizer::{AozoraToken, TextToken, TextKind};

    fn make_text(s: &str) -> ParsedItem {
        ParsedItem::Text(DecoratedText {
            text: s.to_string(),
            ruby: None,
            span: Span::default(),
        })
    }

    #[test]
    fn test_simple_items() {
        let items = vec![make_text("abc"), make_text("def")];
        let root = parse_blocks(items).unwrap();
        
        assert_eq!(root.decoration, None);
        assert_eq!(root.elements.len(), 2);
        if let BlockElement::Item(ParsedItem::Text(t)) = &root.elements[0] {
            assert_eq!(t.text, "abc");
        } else {
            panic!("Expected item");
        }
    }

    #[test]
    fn test_nested_block() {
        // [Begin, text, End]
        let items = vec![
            ParsedItem::Command { cmd: Command::CommandBegin(CommandBegin::Alignment(Alignment { is_upper: true, space: 1 })), span: Span::new(0, 10) },
            make_text("indented"),
            ParsedItem::Command { cmd: Command::CommandEnd(CommandEnd::Alignment), span: Span::new(18, 28) },
        ];

        let root = parse_blocks(items).unwrap();
        assert_eq!(root.elements.len(), 1);
        
        if let BlockElement::Block(b) = &root.elements[0] {
             assert!(matches!(b.decoration, Some(CommandBegin::Alignment(_))));
             assert_eq!(b.elements.len(), 1);
             if let BlockElement::Item(ParsedItem::Text(t)) = &b.elements[0] {
                 assert_eq!(t.text, "indented");
             }
        } else {
            panic!("Expected block");
        }
    }
    
    #[test]
    fn test_deep_nesting() {
        // Root -> Block1 -> Block2 -> Item
        let items = vec![
            ParsedItem::Command { cmd: Command::CommandBegin(CommandBegin::Yokogumi), span: Span::new(0, 5) },
            ParsedItem::Command { cmd: Command::CommandBegin(CommandBegin::Kakomikei), span: Span::new(5, 10) },
            make_text("Deep"),
            ParsedItem::Command { cmd: Command::CommandEnd(CommandEnd::Kakomikei), span: Span::new(14, 20) },
            ParsedItem::Command { cmd: Command::CommandEnd(CommandEnd::Yokogumi), span: Span::new(20, 25) },
        ];
        
        let root = parse_blocks(items).unwrap();
        if let BlockElement::Block(b1) = &root.elements[0] {
            assert!(matches!(b1.decoration, Some(CommandBegin::Yokogumi)));
            if let BlockElement::Block(b2) = &b1.elements[0] {
                assert!(matches!(b2.decoration, Some(CommandBegin::Kakomikei)));
                if let BlockElement::Item(ParsedItem::Text(t)) = &b2.elements[0] {
                     assert_eq!(t.text, "Deep");
                }
            }
        }
    }

    #[test]
    fn test_unclosed_auto_close() {
         // Unclosed blocks should be auto-closed at document end
         let items = vec![
            ParsedItem::Command { cmd: Command::CommandBegin(CommandBegin::Yokogumi), span: Span::new(0, 5) },
            make_text("oops"),
        ];
        let root = parse_blocks(items).unwrap();
        // The unclosed block should be added to root
        assert_eq!(root.elements.len(), 1);
        if let BlockElement::Block(b) = &root.elements[0] {
            assert!(matches!(b.decoration, Some(CommandBegin::Yokogumi)));
            assert_eq!(b.elements.len(), 1);
        } else {
            panic!("Expected block");
        }
    }

    #[test]
    fn test_unexpected_end_error() {
         let items = vec![
            ParsedItem::Command { cmd: Command::CommandEnd(CommandEnd::Yokogumi), span: Span::default() },
        ];
        let res = parse_blocks(items);
        assert!(matches!(res, Err(BlockParseError::UnexpectedEnd { .. })));
    }
}
