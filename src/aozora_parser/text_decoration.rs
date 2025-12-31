use itertools::Itertools;

use crate::aozora_parser::tokenizer::{AozoraToken, CommandToken, TextToken};

enum TextDecoration {
    Ruby(String),
    Bouten,
    Superscript,
    Subscript,
    TateNakaYoko,
}

struct AozoraText {
    text: String,
    decoration: Option<TextDecoration>,
    jisage: Option<usize>,
}

enum ParseError {
    UnexpectedToken(AozoraToken),
}

enum TextDecoratedTokens {
    AozoraText(AozoraText),
    Command(CommandToken),

    Newline,

    Odoriji,
    DakutenOdoriji,
}

fn parse(tokens: Vec<AozoraToken>) -> Result<Vec<TextDecoratedTokens>, ParseError> {
    let mut tokens = tokens.iter().multipeek();
    let mut ruby_buffer: Vec<TextToken> = Vec::new();
    let mut token_buffer: Vec<TextDecoratedTokens> = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
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
                token_buffer.push(TextDecoratedTokens::AozoraText(AozoraText {
                    text: ruby_buffer.iter().map(|t| t.content.clone()).join(""),
                    decoration: Some(TextDecoration::Ruby(ruby)),
                    jisage: None,
                }));
            }
            _ => {}
        }
    }
    Err(ParseError::UnexpectedToken(AozoraToken::Newline))
}
