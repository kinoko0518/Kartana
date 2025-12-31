use itertools::Itertools;

fn is_hiragana(c: char) -> bool {
    (0x3040 <= (c as u32)) && ((c as u32) <= 0x309F)
}

fn is_katakana(c: char) -> bool {
    (0x30A0 <= (c as u32)) && ((c as u32) <= 0x30FF)
}

fn is_kanji(c: char) -> bool {
    match c {
        '々' => true,
        '〆' => true,
        '〇' => true,
        'ヶ' => true,
        '仝' => true,
        c if ('\u{4E00}'..='\u{9FFF}').contains(&c) => true,
        c if ('\u{3400}'..='\u{4DBF}').contains(&c) => true,
        c if ('\u{F900}'..='\u{FAFF}').contains(&c) => true,
        c if ('\u{20000}'..='\u{2A6DF}').contains(&c) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TextKind {
    Hiragana,
    Katana,
    Kanji,
    Other,
}

#[derive(Debug, Clone)]
struct TextToken {
    content: String,
    kind: TextKind,
}

#[derive(Debug, Clone)]
struct CommandToken {
    content: String,
}

#[derive(Debug, Clone)]
enum AozoraToken {
    Text(TextToken),

    Newline,
    RubyStart,
    RubyEnd,
    RubySeparator,

    Command(CommandToken),

    Odoriji,
    DakutenOdoriji,
}

enum TokenizeError {
    UnclosedCommand,
}

fn parse_aozora(text: String) -> Result<Vec<AozoraToken>, TokenizeError> {
    let mut tokens = Vec::new();
    let mut text = text.chars().multipeek();
    let mut buffer = String::new();

    while let Some(c) = text.next() {
        text.reset_peek();
        match c {
            '《' => {
                tokens.push(AozoraToken::RubyStart);
            }
            '》' => {
                tokens.push(AozoraToken::RubyEnd);
            }
            '｜' => {
                tokens.push(AozoraToken::RubySeparator);
            }
            '\n' => {
                tokens.push(AozoraToken::Newline);
            }
            c if (c, text.peek(), text.peek()) == ('／', Some(&'″'), Some(&'＼')) => {
                tokens.push(AozoraToken::DakutenOdoriji);
                text.next();
                text.next();
                text.reset_peek();
            }
            c if (c, text.peek()) == ('／', Some(&'＼')) => {
                tokens.push(AozoraToken::Odoriji);
                text.next();
                text.reset_peek();
            }
            c if (c, text.peek()) == ('［', Some(&'＃')) => {
                text.next(); // '＃'を消費
                let mut buffer = String::new();
                tokens.push(loop {
                    if let Some(c) = text.next() {
                        if c == '］' {
                            break AozoraToken::Command(CommandToken { content: buffer });
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        return Err(TokenizeError::UnclosedCommand);
                    }
                });
            }
            c if is_kanji(c) => {
                let mut buffer = String::new();
                while let Some(c2) = text.next()
                    && is_kanji(c2)
                {
                    buffer.push(c2);
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Kanji,
                }));
            }
            c if is_hiragana(c) => {
                let mut buffer = String::new();
                while let Some(c2) = text.next()
                    && is_hiragana(c2)
                {
                    buffer.push(c2);
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Hiragana,
                }));
            }
            c if is_katakana(c) => {
                let mut buffer = String::new();
                while let Some(c2) = text.next()
                    && is_katakana(c2)
                {
                    buffer.push(c2);
                }
                tokens.push(AozoraToken::Text(TextToken {
                    content: buffer,
                    kind: TextKind::Katana,
                }));
            }
            _ => {
                let mut buffer = String::new();
                while let Some(c2) = text.next()
                    && !is_kanji(c2)
                    && !is_hiragana(c2)
                    && !is_katakana(c2)
                {
                    buffer.push(c2);
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
