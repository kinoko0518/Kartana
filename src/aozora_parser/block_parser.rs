use crate::aozora_parser::parser::ParsedItem;
use crate::aozora_parser::tokenizer::command::{Command, CommandBegin, CommandEnd};

#[derive(Debug, PartialEq, Clone)]
pub enum BlockElement {
    Item(ParsedItem),
    Block(AozoraBlock),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AozoraBlock {
    pub decoration: Option<CommandBegin>, // None for Root
    pub elements: Vec<BlockElement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockParseError {
    UnexpectedEnd(CommandEnd),
    UnclosedBlock(CommandBegin),
}

pub fn parse_blocks(items: Vec<ParsedItem>) -> Result<AozoraBlock, BlockParseError> {
    let mut stack: Vec<AozoraBlock> = Vec::new();
    // Root block
    stack.push(AozoraBlock {
        decoration: None,
        elements: Vec::new(),
    });

    for item in items {
        if let ParsedItem::Command(Command::CommandBegin(begin)) = &item {
            // Start a new block
            let new_block = AozoraBlock {
                decoration: Some(begin.clone()),
                elements: Vec::new(),
            };
            stack.push(new_block);
        } else if let ParsedItem::Command(Command::CommandEnd(end)) = &item {
            // Close the current block
            if stack.len() <= 1 {
                // Trying to close root or extra closing tag
                return Err(BlockParseError::UnexpectedEnd(end.clone()));
            }

            let finished_block = stack.pop().unwrap();
            
            // Validate if closing tag matches opening tag (this logic can be refined)
            // For now, let's just assume simple nesting or check basic type compatibility if needed.
            // But CommandEnd does not always carry same data as CommandBegin (e.g. Jitsume(size) vs Jitsume).
            // Let's implement a loose check or strict check?
            // User requirement didn't specify strict validation, but it's good practice.
            // However, implementing `matches` helper is better.
            
            // For now, just add to parent.
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

    if stack.len() != 1 {
        // Unclosed blocks remain
        let unclosed = stack.pop().unwrap();
        // If it's not root (which we checked len != 1), it has decoration
        if let Some(dec) = unclosed.decoration {
             return Err(BlockParseError::UnclosedBlock(dec));
        } else {
             // Should not happen as root is bottom
             panic!("Stack logic error");
        }
    }

    Ok(stack.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aozora_parser::parser::{DecoratedText, SpecialCharacter};
    use crate::aozora_parser::tokenizer::command::{
        Alignment, Midashi, MidashiSize, MidashiType, SingleCommand,
    };
    use crate::aozora_parser::tokenizer::{AozoraToken, TextToken, TextKind};

    fn make_text(s: &str) -> ParsedItem {
        ParsedItem::Text(DecoratedText {
            text: s.to_string(),
            ruby: None,
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
            ParsedItem::Command(Command::CommandBegin(CommandBegin::Alignment(Alignment { is_upper: true, space: 1 }))),
            make_text("indented"),
            ParsedItem::Command(Command::CommandEnd(CommandEnd::Alignment)),
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
            ParsedItem::Command(Command::CommandBegin(CommandBegin::Yokogumi)),
            ParsedItem::Command(Command::CommandBegin(CommandBegin::Kakomikei)),
            make_text("Deep"),
            ParsedItem::Command(Command::CommandEnd(CommandEnd::Kakomikei)),
            ParsedItem::Command(Command::CommandEnd(CommandEnd::Yokogumi)),
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
    fn test_unclosed_error() {
         let items = vec![
            ParsedItem::Command(Command::CommandBegin(CommandBegin::Yokogumi)),
            make_text("oops"),
        ];
        let res = parse_blocks(items);
        assert!(matches!(res, Err(BlockParseError::UnclosedBlock(_))));
    }

    #[test]
    fn test_unexpected_end_error() {
         let items = vec![
            ParsedItem::Command(Command::CommandEnd(CommandEnd::Yokogumi)),
        ];
        let res = parse_blocks(items);
        assert!(matches!(res, Err(BlockParseError::UnexpectedEnd(_))));
    }
}
