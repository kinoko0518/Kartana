use crate::aozora_parser::block_parser::AozoraBlock;
use crate::aozora_parser::xhtml_generator::{XhtmlGenerator, TocEntry};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write;
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
        let (body_content, toc_entries) = XhtmlGenerator::generate(&self.blocks, &self.title);

        // META-INF/container.xml
        zip.start_file("META-INF/container.xml", options_deflate)?;
        zip.write_all(self.generate_container().as_bytes())?;

        // item/standard.opf
        zip.start_file("item/standard.opf", options_deflate)?;
        zip.write_all(self.generate_opf().as_bytes())?;

        // item/nav.xhtml
        zip.start_file("item/nav.xhtml", options_deflate)?;
        zip.write_all(self.generate_nav(&toc_entries).as_bytes())?;
        
        // Copy CSS files from reference directory
        zip.add_directory("item/style", options_deflate)?;
        let css_files = self.get_css_contents();
        for (filename, content) in &css_files {
            zip.start_file(format!("item/style/{}", filename), options_deflate)?;
            zip.write_all(content.as_bytes())?;
        }

        // item/xhtml/title.xhtml (title page)
        zip.add_directory("item/xhtml", options_deflate)?;
        zip.start_file("item/xhtml/title.xhtml", options_deflate)?;
        zip.write_all(self.generate_title_page().as_bytes())?;

        // item/xhtml/0001.xhtml (main content)
        zip.start_file("item/xhtml/0001.xhtml", options_deflate)?;
        zip.write_all(body_content.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    fn generate_container(&self) -> String {
        include_str!("epub_template/container.xml").to_string()
    }

    fn generate_opf(&self) -> String {
        include_str!("epub_template/standard.opf")
            .replace("{title}", &self.title)
            .replace("{creator}", &self.creator)
            .replace("{uuid}", &self.uuid)
            .replace("{modified}", &chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }

    fn generate_title_page(&self) -> String {
        include_str!("epub_template/title.xhtml")
            .replace("{title}", &self.title)
            .replace("{creator}", &self.creator)
    }

    fn generate_nav(&self, toc: &[TocEntry]) -> String {
        let mut toc_items = String::new();
        
        // Add title page link first
        writeln!(toc_items, "\t\t\t<li><a href=\"xhtml/title.xhtml\">{}</a>", self.title).unwrap();
        
        // Add heading links
        if !toc.is_empty() {
            toc_items.push_str("\t\t<ol>\n");
            for entry in toc {
                writeln!(toc_items, "\t\t\t<li><a href=\"xhtml/0001.xhtml#{}\">　{}</a></li>", entry.id, entry.text).unwrap();
            }
            toc_items.push_str("\t\t</ol>\n");
        }
        toc_items.push_str("\t\t</li>");

        include_str!("epub_template/nav.xhtml")
            .replace("{title}", &self.title)
            .replace("{toc_items}", &toc_items)
    }

    fn get_css_contents(&self) -> Vec<(String, String)> {
        // CSS files embedded from src/aozora_parser/epub_template/css/
        let css_files = [
            ("aozora.css", include_str!("epub_template/css/aozora.css")),
            ("book-style.css", include_str!("epub_template/css/book-style.css")),
            ("fixed-layout-jp.css", include_str!("epub_template/css/fixed-layout-jp.css")),
            ("font.css", include_str!("epub_template/css/font.css")),
            ("style-advance.css", include_str!("epub_template/css/style-advance.css")),
            ("style-reset.css", include_str!("epub_template/css/style-reset.css")),
            ("style-standard.css", include_str!("epub_template/css/style-standard.css")),
            ("text.css", include_str!("epub_template/css/text.css")),
        ];
        
        css_files.iter().map(|(name, content)| (name.to_string(), content.to_string())).collect()
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
