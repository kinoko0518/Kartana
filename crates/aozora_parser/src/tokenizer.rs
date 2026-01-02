pub mod command;

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

/// 元テキスト内での位置情報（文字単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// 開始位置（0-indexed、文字単位）
    pub start: usize,
    /// 終了位置（exclusive、0-indexed、文字単位）
    pub end: usize,
}

impl Span {
    /// 新しいSpanを作成
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// 2つのSpanを結合（最小startから最大endまで）
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextKind {
    Hiragana,
    Katakana,
    Kanji,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextToken {
    pub content: String,
    pub kind: TextKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandToken {
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AozoraToken {
    Text(TextToken),

    Ruby { content: String, span: Span },
    RubySeparator(Span),

    Command(CommandToken),

    Newline(Span),

    Odoriji(Span),
    DakutenOdoriji(Span),
}

#[derive(Debug, Clone)]
pub enum TokenizeError {
    UnclosedCommand(Span),
}

pub fn parse_aozora(text: String) -> Result<Vec<AozoraToken>, TokenizeError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut pos: usize = 0; // 現在の文字位置

    while pos < chars.len() {
        let c = chars[pos];
        match c {
            '《' => {
                let start = pos;
                pos += 1; // '《'を消費
                let mut buffer = String::new();
                while pos < chars.len() {
                    let c2 = chars[pos];
                    pos += 1;
                    if c2 == '》' {
                        break;
                    } else {
                        buffer.push(c2);
                    }
                }
                tokens.push(AozoraToken::Ruby {
                    content: buffer,
                    span: Span::new(start, pos),
                });
            }
            '｜' => {
                tokens.push(AozoraToken::RubySeparator(Span::new(pos, pos + 1)));
                pos += 1;
            }
            '\n' => {
                tokens.push(AozoraToken::Newline(Span::new(pos, pos + 1)));
                pos += 1;
            }
            '／' => {
                let start = pos;
                let p1 = chars.get(pos + 1).cloned();
                let p2 = chars.get(pos + 2).cloned();

                match (p1, p2) {
                    (Some('″'), Some('＼')) => {
                        // 濁点踊り字 ／″＼
                        tokens.push(AozoraToken::DakutenOdoriji(Span::new(start, start + 3)));
                        pos += 3;
                    }
                    (Some('＼'), _) => {
                        // 踊り字 ／＼
                        tokens.push(AozoraToken::Odoriji(Span::new(start, start + 2)));
                        pos += 2;
                    }
                    _ => {
                        let mut buffer = String::new();
                        buffer.push('／');
                        pos += 1;
                        while pos < chars.len() && is_other(chars[pos]) {
                            buffer.push(chars[pos]);
                            pos += 1;
                        }
                        tokens.push(AozoraToken::Text(TextToken {
                            content: buffer,
                            kind: TextKind::Other,
                            span: Span::new(start, pos),
                        }));
                    }
                }
            }
            '［' if chars.get(pos + 1) == Some(&'＃') => {
                let start = pos;
                // '［'と'＃'を消費
                pos += 2;
                let mut buffer = String::new();
                loop {
                    let c = chars.get(pos);
                    match c {
                        Some(&'］') => {
                            tokens.push(AozoraToken::Command(CommandToken {
                                content: buffer,
                                span: Span::new(start, pos),
                            }));
                            break;
                        }
                        c if c.is_none() | c.is_some_and(|c| (*c).is_whitespace()) => {
                            // ルビに空白文字は入り得ないため、
                            // 閉じられなかったと判定する
                            return Err(TokenizeError::UnclosedCommand(Span::new(start, pos)));
                        }
                        otherwise => {
                            // is_noneでotherwiseがNoneでないことは確認済み
                            buffer.push(*otherwise.unwrap());
                        }
                    }
                    pos += 1;
                }
            }
            c if is_kanji(c) => {
                let start = pos;
                let mut buffer = String::new();
                buffer.push(c);
                pos += 1;
                while pos < chars.len() && is_kanji(chars[pos]) {
                    buffer.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Kanji,
                    span: Span::new(start, pos),
                }));
            }
            c if is_hiragana(c) => {
                let start = pos;
                let mut buffer = String::new();
                buffer.push(c);
                pos += 1;
                while pos < chars.len() && is_hiragana(chars[pos]) {
                    buffer.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Hiragana,
                    span: Span::new(start, pos),
                }));
            }
            c if is_katakana(c) => {
                let start = pos;
                let mut buffer = String::new();
                buffer.push(c);
                pos += 1;
                while pos < chars.len() && is_katakana(chars[pos]) {
                    buffer.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Katakana,
                    span: Span::new(start, pos),
                }));
            }
            _ => {
                let start = pos;
                let mut buffer = String::new();
                buffer.push(c);
                pos += 1;
                while pos < chars.len() && is_other(chars[pos]) {
                    buffer.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Other,
                    span: Span::new(start, pos),
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
                assert_eq!(t.kind, TextKind::Katakana);
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
            AozoraToken::Ruby { content, span } => {
                assert_eq!(content, "かんじ");
                assert_eq!(span.start, 2); // 漢字 = 2 chars
                assert_eq!(span.end, 7); // 《かんじ》 = 5 chars
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
        assert!(matches!(tokens[0], AozoraToken::Odoriji(_)));
    }

    #[test]
    fn test_dakuten_odoriji() {
        let input = "／″＼".to_string();
        let tokens = parse_aozora(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], AozoraToken::DakutenOdoriji(_)));
    }
}
