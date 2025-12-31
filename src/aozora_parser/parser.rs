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
mod tests;
