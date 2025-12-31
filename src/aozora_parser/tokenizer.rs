mod command;

use itertools::Itertools;

fn is_hiragana(c: char) -> bool {
    (0x3040 <= (c as u32)) && ((c as u32) <= 0x309F)
}

fn is_katakana(c: char) -> bool {
    (0x30A0 <= (c as u32)) && ((c as u32) <= 0x30FF)
}

fn is_kanji(c: char) -> bool {
    match c {
        '々' | '〆' | '〇' | 'ヶ' | '仝' => true,
        c if ('\u{4E00}'..='\u{9FFF}').contains(&c) => true,
        c if ('\u{3400}'..='\u{4DBF}').contains(&c) => true,
        c if ('\u{F900}'..='\u{FAFF}').contains(&c) => true,
        c if ('\u{20000}'..='\u{2A6DF}').contains(&c) => true,
        _ => false,
    }
}

fn is_other(c: char) -> bool {
    !is_kanji(c)
        && !is_hiragana(c)
        && !is_katakana(c)
        && c != '《'
        && c != '》'
        && c != '｜'
        && c != '\n'
        && c != '［'
        && c != '／'
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextKind {
    Hiragana,
    Katana,
    Kanji,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextToken {
    pub content: String,
    pub kind: TextKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandToken {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AozoraToken {
    Text(TextToken),

    Ruby(String),
    RubySeparator,

    Command(CommandToken),

    Newline,

    Odoriji,
    DakutenOdoriji,
}

#[derive(Debug, Clone)]
pub enum TokenizeError {
    UnclosedCommand,
}

pub fn parse_aozora(text: String) -> Result<Vec<AozoraToken>, TokenizeError> {
    let mut tokens = Vec::new();
    let mut text = text.chars().multipeek();

    while let Some(c) = text.next() {
        match c {
            '《' => {
                let mut buffer = String::new();
                while let Some(c2) = text.next() {
                    if c2 == '》' {
                        break;
                    } else {
                        buffer.push(c2);
                    }
                }
                tokens.push(AozoraToken::Ruby(buffer));
            }
            '｜' => {
                tokens.push(AozoraToken::RubySeparator);
            }
            '\n' => {
                tokens.push(AozoraToken::Newline);
            }
            '／' => {
                text.reset_peek();
                let p1 = text.peek().cloned();
                let p2 = text.peek().cloned();

                match (p1, p2) {
                    (Some('″'), Some('＼')) => {
                        // 濁点踊り字
                        tokens.push(AozoraToken::DakutenOdoriji);
                        text.next(); // Consume '″'
                        text.next(); // Consume '＼'
                    }
                    (Some('＼'), _) => {
                        // 踊り字
                        tokens.push(AozoraToken::Odoriji);
                        text.next(); // Consume '＼'
                    }
                    _ => {
                        let mut buffer = String::new();
                        buffer.push('／');
                        while let Some(pc) = text.peek() {
                            let pc = *pc;
                            if is_other(pc) {
                                buffer.push(pc);
                                text.next();
                            } else {
                                break;
                            }
                        }
                        tokens.push(AozoraToken::Text(TextToken {
                            content: buffer,
                            kind: TextKind::Other,
                        }));
                    }
                }
            }
            '［' => {
                // Check for command starter '＃'
                text.reset_peek();
                if let Some(&'＃') = text.peek() {
                    text.next(); // Consume '＃'
                    let mut buffer = String::new();
                    loop {
                        if let Some(c) = text.next() {
                            if c == '］' {
                                tokens.push(AozoraToken::Command(CommandToken { content: buffer }));
                                break;
                            } else {
                                buffer.push(c);
                            }
                        } else {
                            return Err(TokenizeError::UnclosedCommand);
                        }
                    }
                } else {
                    // Just a '［', treat as Other text
                    let mut buffer = String::new();
                    buffer.push('［');
                    while let Some(pc) = text.peek() {
                        let pc = *pc;
                        if is_other(pc) {
                            buffer.push(pc);
                            text.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(AozoraToken::Text(TextToken {
                        content: buffer,
                        kind: TextKind::Other,
                    }));
                }
            }
            c if is_kanji(c) => {
                let mut buffer = String::new();
                buffer.push(c);
                text.reset_peek();
                while let Some(&c2) = text.peek() {
                    if is_kanji(c2) {
                        buffer.push(c2);
                        text.next();
                    } else {
                        break;
                    }
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Kanji,
                }));
            }
            c if is_hiragana(c) => {
                let mut buffer = String::new();
                buffer.push(c);
                text.reset_peek();
                while let Some(&c2) = text.peek() {
                    if is_hiragana(c2) {
                        buffer.push(c2);
                        text.next();
                    } else {
                        break;
                    }
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Hiragana,
                }));
            }
            c if is_katakana(c) => {
                let mut buffer = String::new();
                buffer.push(c);
                text.reset_peek();
                while let Some(&c2) = text.peek() {
                    if is_katakana(c2) {
                        buffer.push(c2);
                        text.next();
                    } else {
                        break;
                    }
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Katana,
                }));
            }
            _ => {
                let mut buffer = String::new();
                buffer.push(c);
                text.reset_peek();
                while let Some(&c2) = text.peek() {
                    if is_other(c2) {
                        buffer.push(c2);
                        text.next();
                    } else {
                        break;
                    }
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Other,
                }));
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hiragana() {
        let input = "あいうえお".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            AozoraToken::Text(t) => {
                assert_eq!(t.content, "あいうえお");
                assert_eq!(t.kind, TextKind::Hiragana);
            }
            _ => panic!("Expected Text token"),
        }
    }

    #[test]
    fn test_mixed_text() {
        let input = "漢字ひらがなカタカナ".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 3);

        match &tokens[0] {
            AozoraToken::Text(t) => {
                assert_eq!(t.content, "漢字");
                assert_eq!(t.kind, TextKind::Kanji);
            }
            _ => panic!("Expected Kanji"),
        }
        match &tokens[1] {
            AozoraToken::Text(t) => {
                assert_eq!(t.content, "ひらがな");
                assert_eq!(t.kind, TextKind::Hiragana);
            }
            _ => panic!("Expected Hiragana"),
        }
        match &tokens[2] {
            AozoraToken::Text(t) => {
                assert_eq!(t.content, "カタカナ");
                assert_eq!(t.kind, TextKind::Katana);
            }
            _ => panic!("Expected Katana"),
        }
    }

    #[test]
    fn test_ruby() {
        let input = "漢字《かんじ》".to_string();
        let tokens = parse_aozora(input).unwrap();
        // Kanji, Ruby
        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            AozoraToken::Text(t) => {
                assert_eq!(t.content, "漢字");
            }
            _ => panic!("Expected Kanji"),
        }
        match &tokens[1] {
            AozoraToken::Ruby(r) => {
                assert_eq!(r, "かんじ");
            }
            _ => panic!("Expected Ruby"),
        }
    }

    #[test]
    fn test_command() {
        let input = "［＃改ページ］".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            AozoraToken::Command(c) => {
                assert_eq!(c.content, "改ページ");
            }
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_odoriji() {
        let input = "／＼".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], AozoraToken::Odoriji));
    }

    #[test]
    fn test_dakuten_odoriji() {
        let input = "／″＼".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], AozoraToken::DakutenOdoriji));
    }
}
