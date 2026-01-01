use dioxus::prelude::*;
use std::fs;
use encoding_rs::SHIFT_JIS;

use crate::decoration::{decorations_from_lint, Decoration};
use crate::rich_editor::RichEditor;
use crate::top_page::works::{ActionIcon, Series};

const BACK_ICON: Asset = asset!("/assets/icons/back.svg");

#[component]
pub fn Editor(series_title: String, chapter_title: String) -> Element {
    let navigator = use_navigator();
    let mut content = use_signal(|| String::new());
    let mut lint_decorations = use_signal(|| Vec::<Decoration>::new());

    // Helper to get file path
    let file_path = {
        let s_title = series_title.clone();
        let c_title = chapter_title.clone();
        move || Series::series_dir(&s_title).join(format!("{}.txt", c_title))
    };

    // Load content on mount
    let file_path_load = file_path.clone();
    use_effect(move || {
        let path = file_path_load();
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let (cow, _, _) = SHIFT_JIS.decode(&bytes);
                // Normalize newlines to \n to avoid mixing \r\n and \n
                let text = cow.replace("\r\n", "\n").replace("\r", "\n");
                content.set(text);
            }
        }
    });

    let mut handle_save = move |_| {
        let text = content();
        
        // Run Linting
        if !text.is_empty() {
             if let Ok(tokens) = aozora_parser::parse_aozora(text.clone()) {
                if let Ok(doc) = aozora_parser::parse(tokens) {
                    if let Ok(block) = aozora_parser::parse_blocks(doc.items) {
                        let result = aozora_parser::lint(block, &text);
                        let decorations = decorations_from_lint(&result.warnings);
                        lint_decorations.set(decorations);
                    }
                }
            }
        } else {
             lint_decorations.set(Vec::new());
        }

        // Convert to CR+LF for Windows/Aozora standard
        // Ensure consistent \r\n by first normalizing to \n
        let text_crlf = text.replace("\r\n", "\n").replace("\r", "\n").replace("\n", "\r\n");
        
        let (cow, _, unmappable) = SHIFT_JIS.encode(&text_crlf);
        if unmappable {
            println!("Warning: unmappable characters found");
        }
        
        let path = file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        if let Err(e) = fs::write(path, cow) {
            println!("Error saving file: {}", e);
        } else {
            println!("File saved successfully");
        }
    };

    // Keybinding Handler for RichEditor (contenteditable)
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let modifiers = evt.modifiers();
        
        // Helper to insert text using execCommand for Undo support
        let insert_text_js = |text: &str| {
            format!(r#"
                const editor = document.getElementById('rich_editor');
                if (editor) {{
                    editor.focus();
                    document.execCommand('insertText', false, "{}");
                }}
            "#, text.replace("\"", "\\\"").replace("\n", "\\n"))
        };

        // Wrap selection: [prefix]selection[suffix]
        let wrap_selection_js = |prefix: &str, suffix: &str, keep_original: bool| {
            format!(r#"
                const editor = document.getElementById('rich_editor');
                if (editor) {{
                    editor.focus();
                    const selection = window.getSelection();
                    if (selection.rangeCount > 0) {{
                        const range = selection.getRangeAt(0);
                        const text = range.toString();
                        const replacement = {} + "{}" + text + "{}";
                        document.execCommand('insertText', false, replacement);
                    }}
                }}
            "#, if keep_original { "text" } else { "\"\"" }, prefix, suffix)
        };

        // Ruby wrap: text《》 with cursor inside
        let ruby_wrap_js = || {
            r#"
                const editor = document.getElementById('rich_editor');
                if (editor) {
                    editor.focus();
                    const selection = window.getSelection();
                    if (selection.rangeCount > 0) {
                        const range = selection.getRangeAt(0);
                        const text = range.toString();
                        const replacement = text + "《》";
                        document.execCommand('insertText', false, replacement);
                        
                        // Move cursor between 《 and 》
                        const newSelection = window.getSelection();
                        if (newSelection.rangeCount > 0) {
                            const newRange = newSelection.getRangeAt(0);
                            newRange.setStart(newRange.endContainer, newRange.endOffset - 1);
                            newRange.collapse(true);
                            newSelection.removeAllRanges();
                            newSelection.addRange(newRange);
                        }
                    }
                }
            "#.to_string()
        };

        let key_str = key.to_string();

        if key_str == "Tab" && !modifiers.shift() && !modifiers.ctrl() && !modifiers.alt() && !modifiers.meta() {
            evt.prevent_default();
            let script = insert_text_js("［＃３字下げ］");
            let _ = document::eval(&script);
        } else if key_str == "Enter" && modifiers.ctrl() {
            evt.prevent_default();
            let script = insert_text_js("\n［＃改頁］");
            let _ = document::eval(&script);
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() && modifiers.shift() {
            evt.prevent_default();
            let script = wrap_selection_js("［＃「", "」に傍点］", true);
            let _ = document::eval(&script);
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() {
            evt.prevent_default();
            let script = ruby_wrap_js();
            let _ = document::eval(&script);
        } else if (key_str == "s" || key_str == "S") && modifiers.ctrl() {
            println!("Ctrl+S pressed, saving...");
            evt.prevent_default();
            evt.stop_propagation();
            handle_save(());
        }
    };

    // Handle content changes from RichEditor
    let handle_change = move |new_text: String| {
        content.set(new_text);
    };

    rsx! {
        div {
            class: "editor_layout",
            // Header / Menu Bar
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
            }

            // Main Content Area
            main {
                class: "editor_content",
                div {
                    class: "text_area_container",
                    RichEditor {
                        content: content,
                        extra_decorations: lint_decorations,
                        onchange: handle_change,
                        onkeydown: handle_keydown,
                    }
                }
            }
        }
    }
}
