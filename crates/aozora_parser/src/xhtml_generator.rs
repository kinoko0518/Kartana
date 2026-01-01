use crate::block_parser::{AozoraBlock, BlockElement};
use crate::parser::{DecoratedText, ParsedItem, SpecialCharacter};
use crate::tokenizer::command::{
    Command, CommandBegin, Midashi, MidashiSize, MidashiType, SingleCommand,
};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub level: u32,
    pub text: String,
    pub id: String,
}

pub struct XhtmlGenerator {
    css: String,
    body: String,
    toc_entries: Vec<TocEntry>,
    next_id: usize,
}

impl XhtmlGenerator {
    pub fn new() -> Self {
        XhtmlGenerator {
            css: String::new(),
            body: String::new(),
            toc_entries: Vec::new(),
            next_id: 1,
        }
    }

    pub fn generate(block: &AozoraBlock, title: &str) -> (String, Vec<TocEntry>) {
        let mut generator = XhtmlGenerator::new();
        generator.render_block(block);

        (
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html
 xmlns="http://www.w3.org/1999/xhtml"
 xmlns:epub="http://www.idpf.org/2007/ops"
 xml:lang="ja"
 class="vrtl"
>
<head>
<meta charset="UTF-8"/>
<title>{}</title>
<link rel="stylesheet" type="text/css" href="../style/book-style.css"/>

</head>
<body>
<div class="main">
{}
</div>
</body>
</html>"#,
                title, generator.body
            ),
            generator.toc_entries,
        )
    }

    fn render_block(&mut self, block: &AozoraBlock) {
        let (tag, classes, close_tag, is_heading) = self.resolve_decoration(&block.decoration);

        // Generate ID if heading
        let id_attr = if is_heading {
            let id = format!("midashi-{}", self.next_id);
            self.next_id += 1;

            // Extract text for TOC
            let toc_text = self.extract_text_from_block(block);

            // Deducing level from tag (h2 -> 2, etc)
            let level = if tag == "h2" {
                2
            } else if tag == "h3" {
                3
            } else if tag == "h4" {
                4
            } else {
                2
            };

            self.toc_entries.push(TocEntry {
                level,
                text: toc_text,
                id: id.clone(),
            });
            format!(" id=\"{}\"", id)
        } else {
            String::new()
        };

        if !tag.is_empty() {
            write!(self.body, "<{}{}", tag, id_attr).unwrap();
            if !classes.is_empty() {
                write!(self.body, " class=\"{}\"", classes.join(" ")).unwrap();
            }
            write!(self.body, ">").unwrap();
        }

        let mut inline_buffer: Vec<&ParsedItem> = Vec::new();

        for elem in &block.elements {
            match elem {
                BlockElement::Item(item) => {
                    match item {
                        ParsedItem::Newline(_) => {
                            if inline_buffer.is_empty() {
                                // Only output empty p if NOT in heading
                                if !is_heading {
                                    write!(self.body, "<p><br/></p>").unwrap();
                                }
                            } else {
                                self.flush_paragraph(&inline_buffer, is_heading);
                                inline_buffer.clear();
                            }
                        }
                        ParsedItem::Command { cmd: Command::CommandBegin(_), .. }
                        | ParsedItem::Command { cmd: Command::CommandEnd(_), .. }
                        | ParsedItem::Command { cmd: Command::SingleCommand(SingleCommand::Midashi(_)), .. } => {
                            // Flush existing buffer
                            self.flush_paragraph(&inline_buffer, is_heading);
                            inline_buffer.clear();

                            // If it is SingleCommand, we must render it now (as block)
                            if let ParsedItem::Command { cmd: Command::SingleCommand(_sc), .. } = item {
                                // Temporarily treat as item
                                // But render_item doesn't wrap in p unless caller does.
                                // We just call render_item and it outputs <h2>...</h2>
                                self.render_item(item);
                            }
                        }
                        _ => {
                            inline_buffer.push(item);
                        }
                    }
                }
                BlockElement::Block(sub_block) => {
                    self.flush_paragraph(&inline_buffer, is_heading);
                    inline_buffer.clear();
                    self.render_block(sub_block);
                }
            }
        }
        self.flush_paragraph(&inline_buffer, is_heading);

        if !close_tag.is_empty() {
            write!(self.body, "{}", close_tag).unwrap();
        }
    }

    fn flush_paragraph(&mut self, buffer: &[&ParsedItem], is_heading: bool) {
        if buffer.is_empty() {
            return;
        }

        // If inside a heading block, DO NOT print <p> tag.
        if !is_heading {
            write!(self.body, "<p>").unwrap();
        }
        for item in buffer {
            self.render_item(item);
        }
        if !is_heading {
            write!(self.body, "</p>").unwrap();
        }
    }

    fn resolve_decoration(
        &self,
        decoration: &Option<CommandBegin>,
    ) -> (String, Vec<String>, String, bool) {
        // Returned tuple: (tag, classes, close_tag, is_heading)
        match decoration {
            None => ("div".to_string(), vec![], "</div>".to_string(), false),
            Some(cmd) => match cmd {
                CommandBegin::Midashi(m) => {
                    let tag = match m.size {
                        MidashiSize::Large => "h2",
                        MidashiSize::Middle => "h3",
                        MidashiSize::Small => "h4",
                    };

                    if let MidashiType::Dogyo = m.kind {
                        (
                            "span".to_string(),
                            vec!["midashi-dogyo".to_string()],
                            "</span>".to_string(),
                            false,
                        )
                    } else {
                        (tag.to_string(), vec![], format!("</{}>", tag), true)
                    }
                }
                CommandBegin::Alignment(a) => {
                    let mut classes = Vec::new();
                    if a.is_upper {
                        classes.push(format!("jisage-{}", a.space));
                    } else {
                        classes.push(format!("chitsuki-{}", a.space));
                    }
                    ("div".to_string(), classes, "</div>".to_string(), false)
                }
                CommandBegin::Kakomikei => (
                    "div".to_string(),
                    vec!["kakomi".to_string()],
                    "</div>".to_string(),
                    false,
                ),
                CommandBegin::Yokogumi => (
                    "div".to_string(),
                    vec!["yokogumi".to_string()],
                    "</div>".to_string(),
                    false,
                ),
                _ => ("div".to_string(), vec![], "</div>".to_string(), false),
            },
        }
    }

    fn extract_text_from_block(&self, block: &AozoraBlock) -> String {
        let mut text = String::new();
        self.accumulate_text_from_block(block, &mut text);
        text
    }

    fn accumulate_text_from_block(&self, block: &AozoraBlock, acc: &mut String) {
        for elem in &block.elements {
            match elem {
                BlockElement::Item(item) => {
                    self.accumulate_text_from_item(item, acc);
                }
                BlockElement::Block(b) => {
                    self.accumulate_text_from_block(b, acc);
                }
            }
        }
    }

    fn accumulate_text_from_item(&self, item: &ParsedItem, acc: &mut String) {
        match item {
            ParsedItem::Text(dt) => acc.push_str(&dt.text),
            ParsedItem::Command { cmd: Command::SingleCommand(SingleCommand::Midashi((_, content))), .. } => {
                acc.push_str(content);
            }
            // Ignore other commands
            _ => {}
        }
    }

    fn render_item(&mut self, item: &ParsedItem) {
        match item {
            ParsedItem::Text(dt) => {
                self.render_text(dt);
            }
            ParsedItem::Newline(_) => {}
            ParsedItem::Command { cmd: Command::SingleCommand(sc), .. } => {
                match sc {
                    SingleCommand::Bold(s) => {
                        write!(self.body, "<span class=\"bold\">{}</span>", escape_html(s))
                            .unwrap();
                    }
                    SingleCommand::Italic(s) => {
                        write!(
                            self.body,
                            "<span class=\"italic\">{}</span>",
                            escape_html(s)
                        )
                        .unwrap();
                    }
                    SingleCommand::Bouten((_, s)) => {
                        write!(self.body, "<span class=\"em\">{}</span>", escape_html(s)).unwrap();
                    }
                    SingleCommand::Bousen((_, s)) => {
                        write!(
                            self.body,
                            "<span class=\"bousen\">{}</span>",
                            escape_html(s)
                        )
                        .unwrap();
                    }
                    SingleCommand::Kaipage => {
                        write!(self.body, "<div class=\"page-break\"></div>").unwrap();
                    }
                    SingleCommand::Kaicho => {
                        write!(self.body, "<div class=\"page-break\"></div>").unwrap();
                    }
                    SingleCommand::Kaimihiraki => {
                        write!(self.body, "<div class=\"kaimihiraki\"></div>").unwrap();
                    }
                    SingleCommand::Kaidan => {
                        write!(self.body, "<div class=\"column-break\"></div>").unwrap();
                    }
                    SingleCommand::Midashi((m, content)) => {
                        let (tag, classes, close, _) =
                            self.resolve_decoration(&Some(CommandBegin::Midashi(m.clone())));

                        // Generate ID for inline midashi too
                        let id = format!("midashi-{}", self.next_id);
                        self.next_id += 1;

                        // Add to TOC
                        let level = if tag == "h2" {
                            2
                        } else if tag == "h3" {
                            3
                        } else if tag == "h4" {
                            4
                        } else {
                            2
                        };
                        self.toc_entries.push(TocEntry {
                            level,
                            text: content.clone(),
                            id: id.clone(),
                        });

                        write!(self.body, "<{} id=\"{}\"", tag, id).unwrap();
                        if !classes.is_empty() {
                            write!(self.body, " class=\"{}\"", classes.join(" ")).unwrap();
                        }
                        write!(self.body, ">").unwrap();
                        write!(self.body, "{}", escape_html(content)).unwrap();
                        write!(self.body, "{}", close).unwrap();
                    }
                    _ => {}
                }
            }
            ParsedItem::SpecialCharacter { kind, .. } => match kind {
                SpecialCharacter::Odoriji => write!(self.body, "／＼").unwrap(),
                SpecialCharacter::DakutenOdoriji => write!(self.body, "／″＼").unwrap(),
            },
            _ => {}
        }
    }

    fn render_text(&mut self, dt: &DecoratedText) {
        let content = escape_html(&dt.text);
        if let Some(ruby) = &dt.ruby {
            write!(
                self.body,
                "<ruby>{}<rt>{}</rt></ruby>",
                content,
                escape_html(ruby)
            )
            .unwrap();
        } else {
            write!(self.body, "{}", content).unwrap();
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_parser::parse_blocks;
    use crate::tokenizer::{self, AozoraToken, Span, TextKind, TextToken};

    #[test]
    fn test_simple_html_generation() {
        let items = vec![ParsedItem::Text(DecoratedText {
            text: "Hello".to_string(),
            ruby: None,
            span: Span::default(),
        })];
        let root = crate::block_parser::parse_blocks(items).unwrap();
        let (html, _) = XhtmlGenerator::generate(&root, "Test");
        assert!(html.contains("Hello"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::block_parser::parse_blocks;
    use crate::parser::parse;
    use crate::tokenizer::command::*;
    use crate::tokenizer::{AozoraToken, parse_aozora};
    use encoding_rs::SHIFT_JIS;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_ningen_shikkaku() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/parser_test_data/桜桃.txt");

        // Read bytes
        let bytes = fs::read(&path).expect("Could not find test file");

        // Decode Shift_JIS
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();

        // Parse
        let tokens = parse_aozora(text).expect("Tokenization failed");
        let doc = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(doc.items).expect("Block parsing failed");

        // Generate
        let (xhtml, _toc) = XhtmlGenerator::generate(&root, "桜桃");

        // Assertions
        assert!(xhtml.contains("子供より親が大事"));
    }

    #[test]
    fn debug_tokens() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/parser_test_data/桜桃.txt");
        let bytes = fs::read(&path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();
        let tokens = parse_aozora(text).expect("Tokenization failed");

        for (i, token) in tokens.iter().enumerate() {
            if let AozoraToken::RubySeparator(_) = token {
                if i + 1 < tokens.len() {
                    match &tokens[i + 1] {
                        AozoraToken::Newline(_) => {
                            println!("Found RubySeparator followed by Newline at index {}", i);
                            let start = if i > 10 { i - 10 } else { 0 };
                            let end = if i + 5 < tokens.len() {
                                i + 5
                            } else {
                                tokens.len()
                            };
                            println!("Context: {:?}", &tokens[start..end]);
                        }
                        AozoraToken::Command(_) => {
                            println!("Found RubySeparator followed by Command at index {}", i);
                            // This might also cause unexpected token if parser expects Text/Ruby
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[test]
    fn test_midashi_html_structure() {
        // ［＃大見出し］見出し［＃大見出し終わり］
        let items = vec![
            ParsedItem::Command { cmd: Command::CommandBegin(CommandBegin::Midashi(Midashi {
                size: MidashiSize::Large,
                kind: MidashiType::Normal,
            })), span: crate::tokenizer::Span::new(0, 8) },
            ParsedItem::Text(DecoratedText {
                text: "見出し".to_string(),
                ruby: None,
                span: crate::tokenizer::Span::new(8, 11),
            }),
            ParsedItem::Command { cmd: Command::CommandEnd(CommandEnd::Midashi(Midashi {
                size: MidashiSize::Large,
                kind: MidashiType::Normal,
            })), span: crate::tokenizer::Span::new(11, 22) },
        ];
        let root = parse_blocks(items).unwrap();
        let (html, toc) = XhtmlGenerator::generate(&root, "Test");
        println!("Generated HTML: {}", html);

        // Validation: H2 should NOT contain p
        assert!(html.contains("<h2 id=\"midashi-1\">見出し</h2>"));
        assert!(!html.contains("<p>見出し</p>"));

        // Validation: TOC
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "見出し");
        assert_eq!(toc[0].level, 2);
        assert_eq!(toc[0].id, "midashi-1");
    }
}
