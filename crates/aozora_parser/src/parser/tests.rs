use super::*;
use crate::tokenizer::{parse_aozora, Span, TextKind, TextToken, AozoraToken};
use std::fs;
use std::path::PathBuf;
use encoding_rs::SHIFT_JIS;

fn with_metadata(tokens: Vec<AozoraToken>) -> Vec<AozoraToken> {
    let mut t = vec![
        AozoraToken::Text(TextToken { content: "Title".to_string(), kind: TextKind::Other, span: Span::new(0, 5) }),
        AozoraToken::Newline(Span::new(5, 6)),
        AozoraToken::Text(TextToken { content: "Author".to_string(), kind: TextKind::Other, span: Span::new(6, 12) }),
        AozoraToken::Newline(Span::new(12, 13)),
    ];
    t.extend(tokens);
    t
}

#[test]
fn debug_hashigaki() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/parser_test_data/桜桃.txt");
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
        AozoraToken::Text(TextToken { content: "こんにちは".to_string(), kind: TextKind::Hiragana, span: Span::new(13, 18) }),
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
        AozoraToken::Text(TextToken { content: "漢字".to_string(), kind: TextKind::Kanji, span: Span::new(13, 15) }),
        AozoraToken::Ruby { content: "かんじ".to_string(), span: Span::new(15, 20) },
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
        AozoraToken::RubySeparator(Span::new(13, 14)),
        AozoraToken::Text(TextToken { content: "ロンドン".to_string(), kind: TextKind::Katakana, span: Span::new(14, 18) }),
        AozoraToken::Text(TextToken { content: "警視庁".to_string(), kind: TextKind::Kanji, span: Span::new(18, 21) }),
        AozoraToken::Ruby { content: "スコットランドヤード".to_string(), span: Span::new(21, 33) },
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
        AozoraToken::RubySeparator(Span::new(13, 14)),
        AozoraToken::Text(TextToken { content: "青空".to_string(), kind: TextKind::Kanji, span: Span::new(14, 16) }),
        AozoraToken::Text(TextToken { content: "文庫".to_string(), kind: TextKind::Kanji, span: Span::new(16, 18) }),
        AozoraToken::Ruby { content: "あおぞらぶんこ".to_string(), span: Span::new(18, 27) },
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
        AozoraToken::Text(TextToken { content: "こんにちは".to_string(), kind: TextKind::Hiragana, span: Span::new(13, 18) }),
        AozoraToken::Text(TextToken { content: "世界".to_string(), kind: TextKind::Kanji, span: Span::new(18, 20) }),
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
        AozoraToken::Text(TextToken { content: "Title".to_string(), kind: TextKind::Other, span: Span::new(0, 5) }),
        AozoraToken::Newline(Span::new(5, 6)),
        AozoraToken::Text(TextToken { content: "Author".to_string(), kind: TextKind::Other, span: Span::new(6, 12) }),
        AozoraToken::Newline(Span::new(12, 13)),
        
        // Start comment block
        AozoraToken::Text(TextToken { content: "-------------------------------------------------------".to_string(), kind: TextKind::Other, span: Span::new(13, 68) }),
        AozoraToken::Newline(Span::new(68, 69)),
        AozoraToken::Text(TextToken { content: "Comment Content".to_string(), kind: TextKind::Other, span: Span::new(69, 84) }),
        AozoraToken::Newline(Span::new(84, 85)),
        AozoraToken::Text(TextToken { content: "-------------------------------------------------------".to_string(), kind: TextKind::Other, span: Span::new(85, 140) }),
        AozoraToken::Newline(Span::new(140, 141)),
        // End comment block

        AozoraToken::Text(TextToken { content: "Body Content".to_string(), kind: TextKind::Other, span: Span::new(141, 153) }),
    ];
    
    // Pass tokens directly as they include metadata lines
    let doc = parse(tokens).unwrap();
    
    assert_eq!(doc.metadata.title, "Title");
    assert_eq!(doc.metadata.author, "Author");
    // Comment block should be skipped. 
    // We expect "Body Content".
    // The newline after the closing separator is consumed.
    
    assert_eq!(doc.items.len(), 1);
    if let ParsedItem::Text(t) = &doc.items[0] {
            assert_eq!(t.text, "Body Content");
    } else {
            panic!("Expected Body Content, got {:?}", doc.items);
    }
}

#[test]
fn debug_outou_block_parse() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/parser_test_data/桜桃.txt");
    let bytes = fs::read(&path).expect("Could not find test file");
    let (cow, _, _) = SHIFT_JIS.decode(&bytes);
    let text = cow.into_owned();
    let tokens = parse_aozora(text).expect("Tokenization failed");
    let doc = parse(tokens).expect("Parsing failed");
    
    // Print out CommandBegin/End items to debug
    for (i, item) in doc.items.iter().enumerate() {
        if let ParsedItem::Command { cmd, .. } = item {
            match cmd {
                crate::tokenizer::command::Command::CommandBegin(b) => {
                    println!("Item {}: CommandBegin({:?})", i, b);
                }
                crate::tokenizer::command::Command::CommandEnd(e) => {
                    println!("Item {}: CommandEnd({:?})", i, e);
                }
                _ => {}
            }
        }
    }
    
    let result = crate::block_parser::parse_blocks(doc.items);
    match result {
        Ok(_) => println!("Block parsing succeeded"),
        Err(e) => {
            println!("Block parsing failed: {:?}", e);
            panic!("Block parsing failed: {:?}", e);
        }
    }
}
