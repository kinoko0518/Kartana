use dioxus::prelude::*;
use crate::top_page::works::{ActionIcon, Series};
use encoding_rs::SHIFT_JIS;
use std::fs;
use std::path::PathBuf;

const BACK_ICON: Asset = asset!("/assets/icons/back.svg");
const PREVIEW_ICON: Asset = asset!("/assets/icons/read.svg");

// --- Hook: use_editor_file ---
#[derive(Clone, Copy, PartialEq)]
pub struct UseEditorFile {
    pub content: Signal<String>,
    pub status: Signal<String>,
    file_path: Signal<PathBuf>,
}

impl UseEditorFile {
    pub fn save(&mut self) {
        let text = (self.content)();
        println!("[use_file] Saving content len: {}", text.len());
        let path = (self.file_path)();

        // Normalize to CRLF for Windows/Aozora standard
        let text_crlf = text
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");

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

// --- Component: Editor ---

#[component]
pub fn Editor(series_title: String, chapter_title: String) -> Element {
    let navigator = use_navigator();
    
    // Custom Hooks
    let mut file = use_editor_file(series_title.clone(), chapter_title.clone());

    // Event Handlers
    let mut handle_save = move |_| {
        file.save();
    };

    let mut handle_change = move |new_text: String| {
        file.content.set(new_text);
    };

    let st = series_title.clone();
    let ct = chapter_title.clone();
    let handle_preview = move |_| {
        file.save();
        navigator.push(format!("/reader/{}/{}", st, ct));
    };

    // Keybinding Handler
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let modifiers = evt.modifiers();
        let key_str = key.to_string();

        if (key_str == "s" || key_str == "S") && modifiers.ctrl() {
            println!("Ctrl+S pressed, saving...");
            evt.prevent_default();
            evt.stop_propagation();
            handle_save(());
        }
    };

    rsx! {
        div {
            class: "editor_layout",
            header {
                class: "editor_header",
                ActionIcon {
                    icon: BACK_ICON,
                    onclick: move |_| navigator.go_back(),
                }
                div {
                    style: "margin-left: auto; color: var(--text-information); font-size: 0.9rem;",
                    "{series_title} - {chapter_title}"
                }
                ActionIcon {
                    icon: PREVIEW_ICON,
                    onclick: handle_preview,
                }
            }

            main {
                class: "editor_content",
                div {
                    class: "text_area_container",
                    div {
                        class: "simple_editor_container",
                        textarea {
                            class: "simple_editor_textarea",
                            value: "{file.content}",
                            oninput: move |evt| handle_change(evt.value()),
                            onkeydown: handle_keydown
                        }
                    }
                }
            }
        }
    }
}
