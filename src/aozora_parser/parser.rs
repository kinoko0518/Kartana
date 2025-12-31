use itertools::Itertools;

use crate::aozora_parser::tokenizer::{self, AozoraToken, CommandToken, TextToken};

#[derive(Debug, PartialEq, Clone)]
pub struct DecoratedText {
    pub text: String,
    pub ruby: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialCharacter {
    Odoriji,
    DakutenOdoriji,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedItem {
    Text(DecoratedText),
    Command(crate::aozora_parser::tokenizer::command::Command),
    Newline,
    SpecialCharacter(SpecialCharacter),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken(AozoraToken),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AozoraMetadata {
    pub title: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AozoraDocument {
    pub metadata: AozoraMetadata,
    pub items: Vec<ParsedItem>,
}

pub fn parse(tokens: Vec<AozoraToken>) -> Result<AozoraDocument, ParseError> {
    let mut tokens_iter = tokens.iter().multipeek();
    
    // Helper to consume a line as String
    let mut consume_line_as_string = || -> String {
        let mut line = String::new();
        while let Some(token) = tokens_iter.peek() {
            match token {
                AozoraToken::Newline => {
                     // Consume newline and break
                     tokens_iter.next(); 
                     break;
                }
                AozoraToken::Text(t) => {
                    line.push_str(&t.content);
                    tokens_iter.next();
                }
                AozoraToken::Ruby(r) => {
                    // Start consuming ruby
                    // Just append ruby text for metadata? 
                    // Or ignore? Usually titles don't have ruby, but if they do, 
                    // represent as "Text(Ruby)"?
                    // For simplicitly, let's append ruby content in parens or just content?
                    // User requirement: "0 lines is Title".
                    // Let's just discard ruby structure and keep text.
                    tokens_iter.next();
                }
                 _ => {
                    // Command, etc. Ignore for metadata string or stringify?
                    // Safe to skip for now 
                    tokens_iter.next();
                }
            }
        }
        line
    };

    let title = consume_line_as_string();
    let author = consume_line_as_string();

    let mut parsed_items: Vec<ParsedItem> = Vec::new();
    let mut ruby_buffer: Vec<TextToken> = Vec::new();

    // Loop through remaining tokens
    let mut in_comment_block = false;

    while let Some(token) = tokens_iter.next() {
        if in_comment_block {
             // Check if this line is a separator to end the block
             // A separator is usually "-------------------------------------------------------"
             match token {
                 AozoraToken::Text(t) => {
                     if t.content.contains("-------------------------------------------------------") {
                         in_comment_block = false;
                         // Consume following newline if present for cleanliness?
                         // The parser loop will handle next newline as usual Item::Newline
                         // But usually separator line ends with newline.
                         // If we are just switching state, the newline after this separator will be parsed as Newline item.
                         // Maybe we want to consume it?
                         if let Some(AozoraToken::Newline) = tokens_iter.peek() {
                             tokens_iter.next();
                         }
                     }
                 }
                 _ => {}
             }
             continue;
        }

        match token {
            AozoraToken::Text(t) => {
                 // Check if this starts a comment block
                 if t.content.contains("-------------------------------------------------------") {
                     in_comment_block = true;
                     // Flush buffer?
                     if !ruby_buffer.is_empty() {
                         parsed_items.push(ParsedItem::Text(DecoratedText {
                            text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                            ruby: None,
                        }));
                        ruby_buffer.clear();
                     }
                     // Skip following newline?
                     if let Some(AozoraToken::Newline) = tokens_iter.peek() {
                         tokens_iter.next();
                     }
                     continue;
                 }
                ruby_buffer.push(t.clone());
            }
            AozoraToken::RubySeparator => {
                // Flush existing buffer first, as | starts a new specific block
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                    }));
                    ruby_buffer.clear();
                }

                let mut temp_buffer: Vec<TextToken> = Vec::new();
                let mut valid_ruby = false;
                
                while let Some(token2) = tokens_iter.peek() {
                    match token2 {
                        AozoraToken::Ruby(r) => {
                            // Success
                            let r_content = r.clone();
                            tokens_iter.next(); // Consume Ruby
                            
                            parsed_items.push(ParsedItem::Text(DecoratedText {
                                text: temp_buffer.iter().map(|t| t.content.clone()).join(""),
                                ruby: Some(r_content),
                            }));
                            valid_ruby = true;
                            break;
                        }
                        AozoraToken::Text(t) => {
                            temp_buffer.push(t.clone());
                            tokens_iter.next(); // Consume Text
                        }
                        _ => {
                            // Unexpected token (Newline, Command, etc.)
                            // Abort Ruby block
                            break;
                        }
                    }
                }
                
                if !valid_ruby {
                    // Treat | as literal text and flush temp buffer
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: "｜".to_string(),
                        ruby: None,
                    }));
                    
                    if !temp_buffer.is_empty() {
                         parsed_items.push(ParsedItem::Text(DecoratedText {
                            text: temp_buffer.iter().map(|t| t.content.clone()).join(""),
                            ruby: None,
                        }));
                    }
                    // The unexpected token is still next in iterator (we peeked), so outer loop will handle it.
                }
            }
            AozoraToken::Ruby(r) => {
                // Ruby without separator applies to the last text token in buffer
                if let Some(last_text) = ruby_buffer.pop() {
                     // Flush any previous tokens in buffer as separate DecoratedText without ruby
                     if !ruby_buffer.is_empty() {
                         parsed_items.push(ParsedItem::Text(DecoratedText {
                             text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                             ruby: None,
                         }));
                         ruby_buffer.clear();
                     }
                     
                     // Push the last token with ruby
                     parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: last_text.content.clone(),
                        ruby: Some(r.clone()),
                    }));
                } else {
                    if r.is_empty() {
                        continue;
                    }
                    eprintln!("Warning: Ruby found without preceding text: {:?}", r);
                }
            }
            AozoraToken::Command(c) => {
                // Flush buffer
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                    }));
                    ruby_buffer.clear();
                }
                if let Some(cmd) = tokenizer::command::parse_command(c.clone()) {
                    // Check for SingleCommand::Midashi referencing previous text
                    let mut merged = false;
                    if let crate::aozora_parser::tokenizer::command::Command::SingleCommand(
                        crate::aozora_parser::tokenizer::command::SingleCommand::Midashi((m, content))
                    ) = &cmd {
                        if let Some(ParsedItem::Text(dt)) = parsed_items.last() {
                            if dt.text == *content {
                                // Match found! Convert to block.
                                let text_item = parsed_items.pop().unwrap(); // Remove pure text
                                
                                parsed_items.push(ParsedItem::Command(
                                    crate::aozora_parser::tokenizer::command::Command::CommandBegin(
                                        crate::aozora_parser::tokenizer::command::CommandBegin::Midashi(m.clone())
                                    )
                                ));
                                parsed_items.push(text_item);
                                parsed_items.push(ParsedItem::Command(
                                    crate::aozora_parser::tokenizer::command::Command::CommandEnd(
                                        crate::aozora_parser::tokenizer::command::CommandEnd::Midashi(m.clone())
                                    )
                                ));
                                merged = true;
                            }
                        }
                    }

                    if !merged {
                        parsed_items.push(ParsedItem::Command(cmd));
                    }
                }
            }
             AozoraToken::Newline => {
                // Flush buffer
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::Newline);
            }
            AozoraToken::Odoriji => {
                 // Flush buffer
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::SpecialCharacter(SpecialCharacter::Odoriji));
            }
            AozoraToken::DakutenOdoriji => {
                 // Flush buffer
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::SpecialCharacter(SpecialCharacter::DakutenOdoriji));
            }
        }
    }
    
    // Final flush
    if !ruby_buffer.is_empty() {
        parsed_items.push(ParsedItem::Text(DecoratedText {
            text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
            ruby: None,
        }));
    }

    Ok(AozoraDocument {
        metadata: AozoraMetadata {
            title,
            author,
        },
        items: parsed_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aozora_parser::tokenizer::{parse_aozora, TextKind, TextToken, AozoraToken};
    use std::fs;
    use std::path::PathBuf;
    use encoding_rs::SHIFT_JIS;

    fn with_metadata(tokens: Vec<AozoraToken>) -> Vec<AozoraToken> {
        let mut t = vec![
            AozoraToken::Text(TextToken { content: "Title".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            AozoraToken::Text(TextToken { content: "Author".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
        ];
        t.extend(tokens);
        t
    }

    #[test]
    fn debug_hashigaki() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/aozora_parser/parser_test_data/人間失格.txt");
        let bytes = fs::read(&path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();
        let tokens = parse_aozora(text).expect("Tokenization failed");
        
        for (i, token) in tokens.iter().enumerate() {
            if let AozoraToken::Text(t) = token {
                if t.content.contains("はしがき") {
                    println!("Found 'はしがき' at index {}", i);
                    let start = if i > 5 { i - 5 } else { 0 };
                    let end = if i + 5 < tokens.len() { i + 5 } else { tokens.len() };
                    println!("Context: {:?}", &tokens[start..end]);
                }
            }
            if let AozoraToken::Command(c) = token {
                 if c.content.contains("はしがき") {
                    println!("Found command 'はしがき' at index {}", i);
                    println!("Context: {:?}", token);
                 }
            }
        }
    }

    #[test]
    fn test_simple_text() {
        let tokens = vec![
            AozoraToken::Text(TextToken { content: "こんにちは".to_string(), kind: TextKind::Hiragana }),
        ];
        let doc = parse(with_metadata(tokens)).unwrap();
        assert_eq!(doc.metadata.title, "Title");
        assert_eq!(doc.metadata.author, "Author");
        assert_eq!(doc.items.len(), 1);
        if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "こんにちは");
            assert_eq!(t.ruby, None);
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_ruby_no_separator() {
        // 漢字《かんじ》
        let tokens = vec![
            AozoraToken::Text(TextToken { content: "漢字".to_string(), kind: TextKind::Kanji }),
            AozoraToken::Ruby("かんじ".to_string()),
        ];
        let doc = parse(with_metadata(tokens)).unwrap();
        assert_eq!(doc.items.len(), 1);
        if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "漢字");
            assert_eq!(t.ruby, Some("かんじ".to_string()));
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_ruby_with_separator() {
        // ｜ロンドン警視庁《スコットランドヤード》
        let tokens = vec![
            AozoraToken::RubySeparator,
            AozoraToken::Text(TextToken { content: "ロンドン".to_string(), kind: TextKind::Katakana }),
            AozoraToken::Text(TextToken { content: "警視庁".to_string(), kind: TextKind::Kanji }),
            AozoraToken::Ruby("スコットランドヤード".to_string()),
        ];
        let doc = parse(with_metadata(tokens)).unwrap();
        assert_eq!(doc.items.len(), 1);
        if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "ロンドン警視庁");
            assert_eq!(t.ruby, Some("スコットランドヤード".to_string()));
        } else {
            panic!("Expected Text");
        }
    }
    
     #[test]
    fn test_ruby_with_separator_multiple_text() {
        // ｜青空文庫《あおぞらぶんこ》
        let tokens = vec![
            AozoraToken::RubySeparator,
            AozoraToken::Text(TextToken { content: "青空".to_string(), kind: TextKind::Kanji }),
            AozoraToken::Text(TextToken { content: "文庫".to_string(), kind: TextKind::Kanji }),
            AozoraToken::Ruby("あおぞらぶんこ".to_string()),
        ];
        let doc = parse(with_metadata(tokens)).unwrap();
        assert_eq!(doc.items.len(), 1);
        if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "青空文庫");
            assert_eq!(t.ruby, Some("あおぞらぶんこ".to_string()));
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_mixed_text_flushing() {
        // こんにちは世界
        let tokens = vec![
            AozoraToken::Text(TextToken { content: "こんにちは".to_string(), kind: TextKind::Hiragana }),
            AozoraToken::Text(TextToken { content: "世界".to_string(), kind: TextKind::Kanji }),
        ];
        let doc = parse(with_metadata(tokens)).unwrap();
        assert_eq!(doc.items.len(), 1); 
        // Should merge into one text item if no ruby/command intervenes
        if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "こんにちは世界");
            assert_eq!(t.ruby, None);
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_comment_block_skipping() {
        let tokens = vec![
            AozoraToken::Text(TextToken { content: "Title".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            AozoraToken::Text(TextToken { content: "Author".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            
            // Start comment block
            AozoraToken::Text(TextToken { content: "-------------------------------------------------------".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            AozoraToken::Text(TextToken { content: "Comment Content".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            AozoraToken::Text(TextToken { content: "-------------------------------------------------------".to_string(), kind: TextKind::Other}),
            AozoraToken::Newline,
            // End comment block

            AozoraToken::Text(TextToken { content: "Body Content".to_string(), kind: TextKind::Other}),
        ];
        
        // Pass tokens directly as they include metadata lines
        let doc = parse(tokens).unwrap();
        
        assert_eq!(doc.metadata.title, "Title");
        assert_eq!(doc.metadata.author, "Author");
        // Comment block should be skipped. 
        // We expect "Body Content".
        // Note: The newline after second separator might be consumed or appearing as Newline item depending on implementation.
        // My implementation: "Usually separator line ends with newline... If we are just switching state, the newline after this separator will be parsed as Newline item. Maybe we want to consume it?"
        // In my code: "if let Some(AozoraToken::Newline) = tokens_iter.peek() { tokens_iter.next(); }"
        // So the newline after the closing separator is consumed.
        
        assert_eq!(doc.items.len(), 1);
        if let ParsedItem::Text(t) = &doc.items[0] {
             assert_eq!(t.text, "Body Content");
        } else {
             panic!("Expected Body Content, got {:?}", doc.items);
        }
    }
}
