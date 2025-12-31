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

pub fn parse(tokens: Vec<AozoraToken>) -> Result<Vec<ParsedItem>, ParseError> {
    let mut tokens = tokens.iter().multipeek();
    let mut ruby_buffer: Vec<TextToken> = Vec::new();
    let mut parsed_items: Vec<ParsedItem> = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
            AozoraToken::Text(t) => {
                ruby_buffer.push(t.clone());
            }
            AozoraToken::RubySeparator => {
                let mut ruby: String = String::new();
                while let Some(token2) = tokens.next() {
                    match token2 {
                        AozoraToken::Ruby(r) => {
                            ruby = r.clone();
                            break;
                        }
                        AozoraToken::Text(t) => {
                            ruby_buffer.push(t.clone());
                        }
                        otherwise => return Err(ParseError::UnexpectedToken(otherwise.clone())),
                    }
                }
                
                // Flush buffer as a single DecoratedText with ruby
                if !ruby_buffer.is_empty() {
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: Some(ruby),
                    }));
                    ruby_buffer.clear();
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
                    // Ruby without preceding text? Ignore or error? 
                    // Aozora spec usually implies ruby follows text. 
                    // If buffer is empty, maybe it's invalid or we just ignore.
                    // Let's assume valid input for now or treat as unexpected?
                    // But wait, if we have just flushed, buffer is empty. 
                    // Ideally tokenizer guarantees context, but here we enforce it.
                    // For robustness, let's treat as error.
                    return Err(ParseError::UnexpectedToken(token.clone()));
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
                    parsed_items.push(ParsedItem::Command(cmd));
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

    Ok(parsed_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aozora_parser::tokenizer::{TextKind, TextToken, AozoraToken};

    #[test]
    fn test_simple_text() {
        let tokens = vec![
            AozoraToken::Text(TextToken { content: "こんにちは".to_string(), kind: TextKind::Hiragana }),
        ];
        let parsed = parse(tokens).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ParsedItem::Text(t) = &parsed[0] {
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
        let parsed = parse(tokens).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ParsedItem::Text(t) = &parsed[0] {
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
        let parsed = parse(tokens).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ParsedItem::Text(t) = &parsed[0] {
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
        let parsed = parse(tokens).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ParsedItem::Text(t) = &parsed[0] {
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
        let parsed = parse(tokens).unwrap();
        assert_eq!(parsed.len(), 1); 
        // Should merge into one text item if no ruby/command intervenes
        if let ParsedItem::Text(t) = &parsed[0] {
            assert_eq!(t.text, "こんにちは世界");
            assert_eq!(t.ruby, None);
        } else {
            panic!("Expected Text");
        }
    }
}
