use crate::aozora_parser::block_parser::AozoraBlock;
use crate::aozora_parser::xhtml_generator::{XhtmlGenerator, TocEntry};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{Write, Cursor};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use uuid::Uuid;

pub struct EpubGenerator {
    title: String,
    creator: String,
    blocks: AozoraBlock,
    uuid: String,
}

impl EpubGenerator {
    pub fn new(title: String, creator: String, blocks: AozoraBlock) -> Self {
        EpubGenerator {
            title,
            creator,
            blocks,
            uuid: Uuid::new_v4().to_string(),
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        // mimetype (must be first, uncompressed)
        zip.start_file("mimetype", options)?;
        zip.write_all(b"application/epub+zip")?;

        let options_deflate = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Generate content first to get TOC
        let (body_content, toc_entries) = XhtmlGenerator::generate(&self.blocks);

        // META-INF/container.xml
        zip.start_file("META-INF/container.xml", options_deflate)?;
        zip.write_all(self.generate_container().as_bytes())?;

        // item/standard.opf
        zip.start_file("item/standard.opf", options_deflate)?;
        zip.write_all(self.generate_opf().as_bytes())?;

        // item/nav.xhtml
        zip.start_file("item/nav.xhtml", options_deflate)?;
        zip.write_all(self.generate_nav(&toc_entries).as_bytes())?;
        
        // item/style/aozora.css (Basic vertical writing CSS)
        zip.add_directory("item/style", options_deflate)?;
        zip.start_file("item/style/aozora.css", options_deflate)?;
        zip.write_all(self.generate_css().as_bytes())?;

        // item/xhtml/content.xhtml
        zip.add_directory("item/xhtml", options_deflate)?;
        zip.start_file("item/xhtml/content.xhtml", options_deflate)?;
        
        zip.write_all(body_content.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    fn generate_container(&self) -> String {
        r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles>
<rootfile full-path="item/standard.opf" media-type="application/oebps-package+xml"/>
</rootfiles>
</container>"#.to_string()
    }

    fn generate_opf(&self) -> String {
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" xml:lang="ja" unique-identifier="unique-id">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title id="title">{}</dc:title>
<dc:creator id="creator">{}</dc:creator>
<dc:language>ja</dc:language>
<dc:identifier id="unique-id">urn:uuid:{}</dc:identifier>
<meta property="dcterms:modified">{}</meta>
</metadata>
<manifest>
<item media-type="application/xhtml+xml" id="nav" href="nav.xhtml" properties="nav"/>
<item id="style" href="style/aozora.css" media-type="text/css"/>
<item id="content" href="xhtml/content.xhtml" media-type="application/xhtml+xml"/>
</manifest>
<spine page-progression-direction="rtl">
<itemref idref="nav"/>
<itemref idref="content"/>
</spine>
</package>"#, 
            self.title, 
            self.creator, 
            self.uuid,
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
        )
    }

    fn generate_nav(&self, toc: &[TocEntry]) -> String {
        let mut nav_body = String::new();
        // Simple ol/li generation. 
        // For nested levels, it's more complex, but standard TOC is often flat or handled by nesting ol.
        // Let's implement flat first or simple nesting?
        // Let's stick to flat list if simple, or respect indentation.
        // For Aozora midashi levels, we can just put them in order.
        
        nav_body.push_str("<ol>\n");
        // Always include "Begin Reading" / Cover? Or just start with headings?
        // If there are no headings, at least link to content.
        if toc.is_empty() {
             nav_body.push_str("<li><a href=\"xhtml/content.xhtml\">本文</a></li>\n");
        } else {
            for entry in toc {
                // Using recursive or stack logic for proper nesting is best, but for MVP:
                // Just use flat list or styling?
                // EPUB TOC usually requires proper nesting for levels.
                // Let's just output flattened for now as indentation might be handled by CSS or reader ignored.
                // Actually, let's try to just output all as li.
                writeln!(nav_body, "<li><a href=\"xhtml/content.xhtml#{}\">{}</a></li>", entry.id, entry.text).unwrap();
            }
        }
        nav_body.push_str("</ol>\n");

        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="ja" xml:lang="ja">
<head>
<meta charset="UTF-8"/>
<title>Navigation</title>
</head>
<body>
<nav epub:type="toc" id="toc">
<h1>目次</h1>
{}
</nav>
</body>
</html>"#, nav_body)
    }

    fn generate_css(&self) -> String {
        r#"@charset "utf-8";
html {
  writing-mode: vertical-rl;
  -webkit-writing-mode: vertical-rl;
  -epub-writing-mode: vertical-rl;
}
body {
  font-family: serif;
}
.jisage-1 { margin-inline-start: 1em; }
.jisage-2 { margin-inline-start: 2em; }
.jisage-3 { margin-inline-start: 3em; }
.chitsuki-1 { margin-block-end: 1em; text-align: right; }
.bousen { text-decoration: underline; text-decoration-style: solid; text-decoration-skip-ink: none; }
.em { font-weight: bold; } /* Simplified */
.kaimihiraki { height: 100vh; width: 100%; break-after: always; }
.page-break { break-after: page; }
.column-break { break-after: column; }
/* ... Add more as needed ... */
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aozora_parser::block_parser::parse_blocks;
    use crate::aozora_parser::tokenizer::parse_aozora;
    use crate::aozora_parser::parser::parse;
    use std::fs;
    use std::path::PathBuf;
    use encoding_rs::SHIFT_JIS;

    #[test]
    fn test_generate_epub_outou() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/aozora_parser/parser_test_data/桜桃.txt");
        let bytes = fs::read(&path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();

        let tokens = parse_aozora(text).expect("Tokenization failed");
        let doc = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(doc.items).expect("Block parsing failed");

        let generator = EpubGenerator::new(
            doc.metadata.title,
            doc.metadata.author,
            root
        );

        let output_path = PathBuf::from("outou.epub");
        generator.write_to_file(&output_path).expect("Failed to write epub");
        
        assert!(output_path.exists());
        
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn generate_outou_test_epub() {
        let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_path.push("src/aozora_parser/parser_test_data/桜桃.txt");
        
        let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_path.push("src/aozora_parser/parser_test_data/桜桃_TEST.epub");

        let bytes = fs::read(&source_path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();

        let tokens = parse_aozora(text).expect("Tokenization failed");
        let doc = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(doc.items).expect("Block parsing failed");

        let generator = EpubGenerator::new(
            doc.metadata.title,
            doc.metadata.author,
            root
        );

        generator.write_to_file(&output_path).expect("Failed to write epub");
        
        assert!(output_path.exists());
    }
}
