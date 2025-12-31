use crate::aozora_parser::block_parser::{AozoraBlock, BlockElement};
use crate::aozora_parser::parser::{DecoratedText, ParsedItem, SpecialCharacter};
use crate::aozora_parser::tokenizer::command::{
    Command, CommandBegin, MidashiSize, MidashiType, SingleCommand,
};
use std::fmt::Write;

pub struct XhtmlGenerator {
    css: String,
    body: String,
}

impl XhtmlGenerator {
    pub fn new() -> Self {
        XhtmlGenerator {
            css: String::new(),
            body: String::new(),
        }
    }

    pub fn generate(block: &AozoraBlock) -> String {
        let mut generator = XhtmlGenerator::new();
        generator.render_block(block);
        
        let css = r#"
@namespace "http://www.w3.org/1999/xhtml";
html {
    writing-mode: vertical-rl;
}
body {
    font-family: serif;
}
.indent-1 { margin-block-start: 1em; }
.indent-2 { margin-block-start: 2em; }
.indent-3 { margin-block-start: 3em; }
.indent-4 { margin-block-start: 4em; }
.indent-5 { margin-block-start: 5em; }
.indent-6 { margin-block-start: 6em; }
.indent-7 { margin-block-start: 7em; }
.indent-8 { margin-block-start: 8em; }
.indent-9 { margin-block-start: 9em; }
.indent-10 { margin-block-start: 10em; }

.chitsuki-1 { margin-block-end: 1em; text-align: right; } /* Assuming vertical-rl, right is bottom */
/* Actually in vertical-rl, block-end is left side of the line for characters? No. 
   writing-mode: vertical-rl;
   block flow is Right to Left.
   inline flow is Top to Bottom.
   
   margin-block-start is Right.
   margin-block-end is Left.
*/

/* Indent usually means Top indentation in vertical writing (inline-start) */
.jisage-1 { margin-inline-start: 1em; }
.jisage-2 { margin-inline-start: 2em; }
.jisage-3 { margin-inline-start: 3em; }
/* ... generic generator needed ... */

.kakomi {
    border: 1px solid currentColor;
    padding: 1em;
    margin: 1em;
}

.yokogumi {
    writing-mode: horizontal-tb;
}

h2, h3, h4, h5 {
    font-weight: bold;
}
.midashi-dogyo {
    /* inline heading */
    display: inline;
    font-weight: bold;
}
"#;
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="ja">
<head>
<meta charset="UTF-8" />
<title>Aozora Output</title>
<style>
{}
</style>
</head>
<body>
{}
</body>
</html>"#,
            css, generator.body
        )
    }

    fn render_block(&mut self, block: &AozoraBlock) {
        let (tag, classes, close_tag) = self.resolve_decoration(&block.decoration);
        
        if !tag.is_empty() {
            write!(self.body, "<{}", tag).unwrap();
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
                        ParsedItem::Newline => {
                            if inline_buffer.is_empty() {
                                // Empty line, output empty p
                                write!(self.body, "<p><br/></p>").unwrap();
                            } else {
                                self.flush_paragraph(&inline_buffer);
                                inline_buffer.clear();
                            }
                        }
                        ParsedItem::Command(Command::CommandBegin(_)) | ParsedItem::Command(Command::CommandEnd(_)) => {
                             self.flush_paragraph(&inline_buffer);
                             inline_buffer.clear();
                        }
                         _ => {
                            inline_buffer.push(item);
                        }
                    }
                }
                BlockElement::Block(sub_block) => {
                    self.flush_paragraph(&inline_buffer);
                    inline_buffer.clear();
                    self.render_block(sub_block);
                }
            }
        }
        self.flush_paragraph(&inline_buffer);

        if !close_tag.is_empty() {
            write!(self.body, "{}", close_tag).unwrap();
        }
    }

    fn flush_paragraph(&mut self, buffer: &[&ParsedItem]) {
        if buffer.is_empty() {
            // Handle empty line as <br/> inside p? Or just empty line? 
            // 0001.xhtml has <p><br/></p> for empty lines.
            // But if buffer is empty and we hit newline, it means we had consecutive newlines.
            // Wait, my loop logic:
            // Text -> push
            // Newline -> flush. If buffer has content, <p>content</p>.
            // If Text, then Newline -> <p>Text</p>.
            // If Newline, then Newline -> buffer empty -> <p><br/></p>?
            // Let's adopt <p><br/></p> for empty buffer to match 0001.xhtml behavior for empty lines (spacing).
            // But wait, if we flushed because of Block, we shouldn't output <p><br/></p> necessarily.
            // Only explicitly for Newline.
            // But here `flush_paragraph` is called on Newline AND Block.
            // Actually, if buffer is empty, it usually means we just finished a block or started.
            // We should NOT output empty p for blocks.
            // But for Newline, `buffer` might be empty if it was `\n\n`.
            // The caller needs to handle this.
            // Let's change logic: `render_block` handles Newline explicitly to emit `<p><br/></p>` or similar?
            // No, simplified logical flow: everything is a paragraph.
            // If buffer is empty, do nothing?
            // But `\n` in Aozora means "End of Line/Para".
            // If we have `Text \n`, we get `flush([Text])`. -> `<p>Text</p>`.
            // If we have `\n` (empty line), we get `flush([])`. -> `<p><br/></p>`?
            // If we have `Block`, we flush. If buffer was empty, we do nothing.
            return;
        }
        
        write!(self.body, "<p>").unwrap();
        for item in buffer {
            self.render_item(item);
        }
        write!(self.body, "</p>").unwrap();
    }

    fn resolve_decoration(&self, decoration: &Option<CommandBegin>) -> (String, Vec<String>, String) {
        match decoration {
            None => ("div".to_string(), vec![], "</div>".to_string()),
            Some(cmd) => match cmd {
                CommandBegin::Midashi(m) => {
                    // Mapping Midashi to h tag
                    // Size: Large -> h2, Middle -> h3, Small -> h4
                    let tag = match m.size {
                        MidashiSize::Large => "h2",
                        MidashiSize::Middle => "h3",
                        MidashiSize::Small => "h4",
                    };
                    
                    if let MidashiType::Dogyo = m.kind {
                         ("span".to_string(), vec!["midashi-dogyo".to_string()], "</span>".to_string())
                    } else {
                         (tag.to_string(), vec![], format!("</{}>", tag))
                    }
                }
                CommandBegin::Alignment(a) => {
                    let mut classes = Vec::new();
                    if a.is_upper {
                        // Jisage -> margin-inline-start
                        classes.push(format!("jisage-{}", a.space));
                    } else {
                        // Chitsuki -> margin-inline-end / text-align end?
                         classes.push(format!("chitsuki-{}", a.space));
                    }
                    ("div".to_string(), classes, "</div>".to_string())
                }
                CommandBegin::Kakomikei => ("div".to_string(), vec!["kakomi".to_string()], "</div>".to_string()),
                CommandBegin::Yokogumi => ("div".to_string(), vec!["yokogumi".to_string()], "</div>".to_string()),
                _ => ("div".to_string(), vec![], "</div>".to_string()), // Fallback
            },
        }
    }

    fn render_item(&mut self, item: &ParsedItem) {
        match item {
            ParsedItem::Text(dt) => {
                self.render_text(dt);
            }
            ParsedItem::Newline => {
                // Newline should be handled by flush_paragraph usually.
                // But inside render_item? No.
                // render_item is called inside flush_paragraph.
                // If Newline is in buffer, it means we treat it as inline br?
                // No, my logic above clears buffer on Newline.
                // So Newline never reaches render_item.
            }
            ParsedItem::Command(Command::SingleCommand(sc)) => {
                match sc {
                     SingleCommand::Bold(s) => {
                         write!(self.body, "<span class=\"bold\">{}</span>", escape_html(s)).unwrap();
                     }
                     SingleCommand::Italic(s) => {
                         write!(self.body, "<span class=\"italic\">{}</span>", escape_html(s)).unwrap();
                     }
                     SingleCommand::Bouten((_, s)) => {
                         write!(self.body, "<span class=\"em\">{}</span>", escape_html(s)).unwrap();
                     }
                     SingleCommand::Bousen((_, s)) => {
                         // Simple underline/overline for now
                         write!(self.body, "<span class=\"bousen\">{}</span>", escape_html(s)).unwrap();
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
                     // Handle embedded Midashi (SingleCommand::Midashi)
                     SingleCommand::Midashi((m, content)) => {
                         let (tag, classes, close) = self.resolve_decoration(&Some(CommandBegin::Midashi(m.clone())));
                         write!(self.body, "<{}", tag).unwrap();
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
            ParsedItem::SpecialCharacter(sc) => {
                match sc {
                    SpecialCharacter::Odoriji => write!(self.body, "／＼").unwrap(),
                    SpecialCharacter::DakutenOdoriji => write!(self.body, "／″＼").unwrap(),
                }
            }
            _ => {}
        }
    }

    fn render_text(&mut self, dt: &DecoratedText) {
        let content = escape_html(&dt.text);
        if let Some(ruby) = &dt.ruby {
            write!(self.body, "<ruby>{}<rt>{}</rt></ruby>", content, escape_html(ruby)).unwrap();
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
    use crate::aozora_parser::block_parser::parse_blocks;
    use crate::aozora_parser::tokenizer::{self, AozoraToken, TextToken, TextKind};

    #[test]
    fn test_simple_html_generation() {
        let items = vec![
            ParsedItem::Text(DecoratedText { text: "Hello".to_string(), ruby: None })
        ];
        let root = crate::aozora_parser::block_parser::parse_blocks(items).unwrap();
        let html = XhtmlGenerator::generate(&root);
        assert!(html.contains("Hello"));
        assert!(html.contains("writing-mode: vertical-rl"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::aozora_parser::block_parser::parse_blocks;
    use crate::aozora_parser::tokenizer::{parse_aozora, AozoraToken};
    use crate::aozora_parser::parser::parse;
    use std::fs;
    use std::path::PathBuf;
    use encoding_rs::SHIFT_JIS;

    #[test]
    fn test_ningen_shikkaku() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/aozora_parser/parser_test_data/人間失格.txt");
        
        // Read bytes
        let bytes = fs::read(&path).expect("Could not find test file");
        
        // Decode Shift_JIS
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();
        
        // Parse
        let tokens = parse_aozora(text).expect("Tokenization failed");
        let items = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(items).expect("Block parsing failed");
        
        // Generate
        let xhtml = XhtmlGenerator::generate(&root);
        
        // Assertions
        assert!(xhtml.contains("私は、その男の写真を三葉、見たことがある。"));
    }

    #[test]
    fn debug_tokens() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/aozora_parser/parser_test_data/人間失格.txt");
        let bytes = fs::read(&path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();
        let tokens = parse_aozora(text).expect("Tokenization failed");
        
        for (i, token) in tokens.iter().enumerate() {
            if let AozoraToken::RubySeparator = token {
                if i + 1 < tokens.len() {
                    match &tokens[i+1] {
                        AozoraToken::Newline => {
                            println!("Found RubySeparator followed by Newline at index {}", i);
                            let start = if i > 10 { i - 10 } else { 0 };
                            let end = if i + 5 < tokens.len() { i + 5 } else { tokens.len() };
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
}
