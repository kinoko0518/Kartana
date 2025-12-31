use crate::aozora_parser::block_parser::AozoraBlock;
use crate::aozora_parser::xhtml_generator::XhtmlGenerator;
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

        // META-INF/container.xml
        zip.start_file("META-INF/container.xml", options_deflate)?;
        zip.write_all(self.generate_container().as_bytes())?;

        // item/standard.opf
        zip.start_file("item/standard.opf", options_deflate)?;
        zip.write_all(self.generate_opf().as_bytes())?;

        // item/nav.xhtml
        zip.start_file("item/nav.xhtml", options_deflate)?;
        zip.write_all(self.generate_nav().as_bytes())?;
        
        // item/style/aozora.css (Basic vertical writing CSS)
        zip.add_directory("item/style", options_deflate)?;
        zip.start_file("item/style/aozora.css", options_deflate)?;
        zip.write_all(self.generate_css().as_bytes())?;

        // item/xhtml/content.xhtml (Single file for now for simplicity, splitting can be added later)
        zip.add_directory("item/xhtml", options_deflate)?;
        zip.start_file("item/xhtml/content.xhtml", options_deflate)?;
        let body_content = XhtmlGenerator::generate(&self.blocks); 
        // Note: XhtmlGenerator generates full HTML with <head>. 
        // We might want to strip it or modify XhtmlGenerator to return body only.
        // Or just use it as is if it fits the requirements. 
        // The requirement said "imitate parser_test_data/人間失格".
        // The sample has multiple XHTML files. 
        // For minimal MPV, we output one file content.xhtml.
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

    fn generate_nav(&self) -> String {
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
<ol>
<li><a href="xhtml/content.xhtml">Begin Reading</a></li>
</ol>
</nav>
</body>
</html>"#)
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
    fn test_generate_epub_ningen_shikkaku() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/aozora_parser/parser_test_data/人間失格.txt");
        let bytes = fs::read(&path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();

        let tokens = parse_aozora(text).expect("Tokenization failed");
        let items = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(items).expect("Block parsing failed");

        let generator = EpubGenerator::new(
            "人間失格".to_string(),
            "太宰治".to_string(),
            root
        );

        let output_path = PathBuf::from("ningen_shikkaku.epub");
        generator.write_to_file(&output_path).expect("Failed to write epub");
        
        // Assert file exists
        assert!(output_path.exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn generate_ningen_shikkaku_test_epub() {
        let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_path.push("src/aozora_parser/parser_test_data/人間失格.txt");
        
        let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_path.push("src/aozora_parser/parser_test_data/人間失格_TEST.epub");

        let bytes = fs::read(&source_path).expect("Could not find test file");
        let (cow, _, _) = SHIFT_JIS.decode(&bytes);
        let text = cow.into_owned();

        let tokens = parse_aozora(text).expect("Tokenization failed");
        let items = parse(tokens).expect("Parsing failed");
        let root = parse_blocks(items).expect("Block parsing failed");

        let generator = EpubGenerator::new(
            "人間失格".to_string(),
            "太宰治".to_string(),
            root
        );

        generator.write_to_file(&output_path).expect("Failed to write epub");
        
        assert!(output_path.exists());
    }
}
