use itertools::Itertools;

use crate::tokenizer::{self, AozoraToken, CommandToken, Span, TextToken};

#[derive(Debug, PartialEq, Clone)]
pub struct DecoratedText {
    pub text: String,
    pub ruby: Option<String>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialCharacter {
    Odoriji,
    DakutenOdoriji,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedItem {
    Text(DecoratedText),
    Command { cmd: crate::tokenizer::command::Command, span: Span },
    Newline(Span),
    SpecialCharacter { kind: SpecialCharacter, span: Span },
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { token: AozoraToken, span: Span },
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
                AozoraToken::Newline(_) => {
                     // Consume newline and break
                     tokens_iter.next(); 
                     break;
                }
                AozoraToken::Text(t) => {
                    line.push_str(&t.content);
                    tokens_iter.next();
                }
                AozoraToken::Ruby { content: _, span: _ } => {
                    // For metadata, just discard ruby structure
                    tokens_iter.next();
                }
                 _ => {
                    // Command, etc. Ignore for metadata string
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

    // Helper to calculate span from ruby_buffer
    fn buffer_span(buffer: &[TextToken]) -> Span {
        if buffer.is_empty() {
            Span::default()
        } else {
            let start = buffer.first().unwrap().span.start;
            let end = buffer.last().unwrap().span.end;
            Span::new(start, end)
        }
    }

    // Loop through remaining tokens
    let mut in_comment_block = false;

    while let Some(token) = tokens_iter.next() {
        if in_comment_block {
             // Check if this line is a separator to end the block
             match token {
                 AozoraToken::Text(t) => {
                     if t.content.contains("-------------------------------------------------------") {
                         in_comment_block = false;
                         if let Some(AozoraToken::Newline(_)) = tokens_iter.peek() {
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
                     // Flush buffer
                     if !ruby_buffer.is_empty() {
                         let span = buffer_span(&ruby_buffer);
                         parsed_items.push(ParsedItem::Text(DecoratedText {
                            text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                            ruby: None,
                            span,
                        }));
                        ruby_buffer.clear();
                     }
                     if let Some(AozoraToken::Newline(_)) = tokens_iter.peek() {
                         tokens_iter.next();
                     }
                     continue;
                 }
                ruby_buffer.push(t.clone());
            }
            AozoraToken::RubySeparator(sep_span) => {
                // Flush existing buffer first, as | starts a new specific block
                if !ruby_buffer.is_empty() {
                    let span = buffer_span(&ruby_buffer);
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                        span,
                    }));
                    ruby_buffer.clear();
                }

                let mut temp_buffer: Vec<TextToken> = Vec::new();
                let mut valid_ruby = false;
                
                while let Some(token2) = tokens_iter.peek() {
                    match token2 {
                        AozoraToken::Ruby { content, span: ruby_span } => {
                            // Success
                            let r_content = content.clone();
                            let r_span = *ruby_span;
                            tokens_iter.next(); // Consume Ruby
                            
                            let text_span = if temp_buffer.is_empty() {
                                *sep_span
                            } else {
                                sep_span.merge(&buffer_span(&temp_buffer))
                            };
                            let full_span = text_span.merge(&r_span);
                            
                            parsed_items.push(ParsedItem::Text(DecoratedText {
                                text: temp_buffer.iter().map(|t| t.content.clone()).join(""),
                                ruby: Some(r_content),
                                span: full_span,
                            }));
                            valid_ruby = true;
                            break;
                        }
                        AozoraToken::Text(t) => {
                            temp_buffer.push((*t).clone());
                            tokens_iter.next(); // Consume Text
                        }
                        _ => {
                            // Unexpected token (Newline, Command, etc.)
                            break;
                        }
                    }
                }
                
                if !valid_ruby {
                    // Treat | as literal text
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: "｜".to_string(),
                        ruby: None,
                        span: *sep_span,
                    }));
                    
                    if !temp_buffer.is_empty() {
                        let span = buffer_span(&temp_buffer);
                        parsed_items.push(ParsedItem::Text(DecoratedText {
                            text: temp_buffer.iter().map(|t| t.content.clone()).join(""),
                            ruby: None,
                            span,
                        }));
                    }
                }
            }
            AozoraToken::Ruby { content, span: ruby_span } => {
                // Ruby without separator applies to the last text token in buffer
                if let Some(last_text) = ruby_buffer.pop() {
                     // Flush any previous tokens in buffer
                     if !ruby_buffer.is_empty() {
                         let span = buffer_span(&ruby_buffer);
                         parsed_items.push(ParsedItem::Text(DecoratedText {
                             text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                             ruby: None,
                             span,
                         }));
                         ruby_buffer.clear();
                     }
                     
                     // Push the last token with ruby
                     let full_span = last_text.span.merge(ruby_span);
                     parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: last_text.content.clone(),
                        ruby: Some(content.clone()),
                        span: full_span,
                    }));
                } else {
                    // Ruby without text - will be detected by Linter
                    if content.is_empty() {
                        continue;
                    }
                }
            }
            AozoraToken::Command(c) => {
                // Flush buffer
                if !ruby_buffer.is_empty() {
                    let span = buffer_span(&ruby_buffer);
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                        span,
                    }));
                    ruby_buffer.clear();
                }
                if let Some(cmd) = tokenizer::command::parse_command(c.clone()) {
                    // Check for SingleCommand::Midashi referencing previous text
                    let mut merged = false;
                    if let crate::tokenizer::command::Command::SingleCommand(
                        crate::tokenizer::command::SingleCommand::Midashi((m, content))
                    ) = &cmd {
                        if let Some(ParsedItem::Text(dt)) = parsed_items.last() {
                            if dt.text == *content {
                                // Match found! Convert to block.
                                let text_item = parsed_items.pop().unwrap();
                                let text_span = if let ParsedItem::Text(dt) = &text_item {
                                    dt.span
                                } else {
                                    Span::default()
                                };
                                
                                parsed_items.push(ParsedItem::Command {
                                    cmd: crate::tokenizer::command::Command::CommandBegin(
                                        crate::tokenizer::command::CommandBegin::Midashi(m.clone())
                                    ),
                                    span: text_span,
                                });
                                parsed_items.push(text_item);
                                parsed_items.push(ParsedItem::Command {
                                    cmd: crate::tokenizer::command::Command::CommandEnd(
                                        crate::tokenizer::command::CommandEnd::Midashi(m.clone())
                                    ),
                                    span: c.span,
                                });
                                merged = true;
                            }
                        }
                    }

                    if !merged {
                        parsed_items.push(ParsedItem::Command { cmd, span: c.span });
                    }
                }
            }
             AozoraToken::Newline(span) => {
                // Flush buffer
                if !ruby_buffer.is_empty() {
                    let buf_span = buffer_span(&ruby_buffer);
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                        span: buf_span,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::Newline(*span));
            }
            AozoraToken::Odoriji(span) => {
                 // Flush buffer
                if !ruby_buffer.is_empty() {
                    let buf_span = buffer_span(&ruby_buffer);
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                        span: buf_span,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::SpecialCharacter { kind: SpecialCharacter::Odoriji, span: *span });
            }
            AozoraToken::DakutenOdoriji(span) => {
                 // Flush buffer
                if !ruby_buffer.is_empty() {
                    let buf_span = buffer_span(&ruby_buffer);
                    parsed_items.push(ParsedItem::Text(DecoratedText {
                        text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                        ruby: None,
                        span: buf_span,
                    }));
                    ruby_buffer.clear();
                }
                parsed_items.push(ParsedItem::SpecialCharacter { kind: SpecialCharacter::DakutenOdoriji, span: *span });
            }
        }
    }
    
    // Final flush
    if !ruby_buffer.is_empty() {
        let span = buffer_span(&ruby_buffer);
        parsed_items.push(ParsedItem::Text(DecoratedText {
            text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
            ruby: None,
            span,
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
