use dioxus::prelude::*;
use std::fs;
use std::path::PathBuf;
use encoding_rs::SHIFT_JIS;
use crate::top_page::works::Series;

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditorFile {
    pub content: Signal<String>,
    pub status: Signal<String>,
    file_path: Signal<PathBuf>,
}

impl UseEditorFile {
    pub fn save(&mut self) {
        let text = (self.content)();
        let path = (self.file_path)();
        
        // Normalize to CRLF for Windows/Aozora standard
        let text_crlf = text.replace("\r\n", "\n").replace("\r", "\n").replace("\n", "\r\n");
        
        let (cow, _, unmappable) = SHIFT_JIS.encode(&text_crlf);
        if unmappable {
            println!("Warning: unmappable characters found");
            self.status.set("Warning: Unmappable chars".to_string());
        }
        
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        if let Err(e) = fs::write(&path, cow) {
            println!("Error saving file: {}", e);
            self.status.set(format!("Error: {}", e));
        } else {
            println!("File saved successfully");
            self.status.set("Saved".to_string());
        }
    }
}

pub fn use_editor_file(series_title: String, chapter_title: String) -> UseEditorFile {
    let mut content = use_signal(|| String::new());
    let mut status = use_signal(|| String::new());
    let mut file_path = use_signal(|| PathBuf::new());

    // Initialize path
    use_hook(move || {
        let path = Series::series_dir(&series_title).join(format!("{}.txt", chapter_title));
        file_path.set(path);
    });

    // Load content on mount
    use_effect(move || {
        let path = file_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let (cow, _, _) = SHIFT_JIS.decode(&bytes);
                // Normalize newlines to \n for internal processing
                let text = cow.replace("\r\n", "\n").replace("\r", "\n");
                content.set(text);
                status.set("Loaded".to_string());
            } else {
                status.set("Error loading".to_string());
            }
        }
    });

    UseEditorFile {
        content,
        status,
        file_path,
    }
}
