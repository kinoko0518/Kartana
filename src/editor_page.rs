use dioxus::prelude::*;
use std::path::PathBuf;
use std::fs;
use encoding_rs::SHIFT_JIS;

use crate::top_page::works::Series;

#[derive(PartialEq, Clone)]
enum MenuTab {
    File,
    Edit,
    Insert,
    Format,
    Tools,
}

#[component]
pub fn Editor(series_title: String, chapter_title: String) -> Element {
    let navigator = use_navigator();
    let mut selected_tab = use_signal(|| MenuTab::Insert); // Default to Insert as it has most features
    let mut content = use_signal(|| String::new());

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
                content.set(cow.into_owned());
            }
        }
    });

    let handle_save = move |_| {
        let text = content();
        // Convert to CR+LF for Windows/Aozora standard
        let text_crlf = text.replace("\n", "\r\n").replace("\r\r\n", "\r\n"); // Simple normalization
        
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

    // Helper for tab class
    let tab_class = |tab: MenuTab| {
        if selected_tab() == tab {
            "menu_item active"
        } else {
            "menu_item"
        }
    };

    // Keybinding Handler
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let modifiers = evt.modifiers();
        
        // Helper to insert text using execCommand for Undo support
        let insert_text_js = |text: &str| {
            format!(r#"
                const textarea = document.getElementById('main_editor');
                if (textarea) {{
                    textarea.focus();
                    // execCommand is deprecated but is the only reliable way to trigger undoable text insertion
                    document.execCommand('insertText', false, "{}");
                }}
            "#, text.replace("\"", "\\\"").replace("\n", "\\n"))
        };

        // Wrap selection: [prefix]selection[suffix]
        let wrap_selection_js = |prefix: &str, suffix: &str, keep_original: bool| {
            format!(r#"
                const textarea = document.getElementById('main_editor');
                if (textarea) {{
                    textarea.focus();
                    const start = textarea.selectionStart;
                    const end = textarea.selectionEnd;
                    const text = textarea.value.substring(start, end);
                    // Construct replacement manually because execCommand inserts at cursor/swaps selection
                    const replacement = {} + "{}" + text + "{}";
                    document.execCommand('insertText', false, replacement);
                    
                    // Restore selection to cover the whole new text (mimicking setRangeText "select" mode)
                    // New length is (maybe text) + prefix + text + suffix
                    // Cursor is currently at the end
                    textarea.setSelectionRange(start, start + replacement.length);
                }}
            "#, if keep_original { "text" } else { "\"\"" }, prefix, suffix)
        };

        // Ruby wrap: |text《》 with cursor inside
        let ruby_wrap_js = || {
            format!(r#"
                const textarea = document.getElementById('main_editor');
                if (textarea) {{
                    textarea.focus();
                    const start = textarea.selectionStart;
                    const end = textarea.selectionEnd;
                    const text = textarea.value.substring(start, end);
                    const replacement = "|" + text + "《》";
                    document.execCommand('insertText', false, replacement);
                    
                    // Move cursor between 《 and 》. 
                    // Current position is at the very end (after 》)
                    const cursor = textarea.selectionEnd - 1; 
                    textarea.setSelectionRange(cursor, cursor);
                }}
            "#)
        };

        // Using safe key string comparison
        let key_str = key.to_string();

        if key_str == "Tab" && !modifiers.shift() && !modifiers.ctrl() && !modifiers.alt() && !modifiers.meta() {
            // Tab -> ［＃３字下げ］
            evt.prevent_default();
            let script = insert_text_js("［＃３字下げ］");
            let _ = document::eval(&script);
        } else if key_str == "Enter" && modifiers.ctrl() {
            // Ctrl+Enter -> ［＃改頁］
            evt.prevent_default();
            let script = insert_text_js("\n［＃改頁］");
            let _ = document::eval(&script);
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() && modifiers.shift() {
            // Ctrl+Shift+< (or Ctrl+Shift+,) -> ［＃「文字列」に傍点］
            evt.prevent_default();
            let script = wrap_selection_js("［＃「", "」に傍点］", true);
            let _ = document::eval(&script);
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() {
            // Ctrl+< (if specific key) or Ctrl+, -> |文字列《（ここにカーソル位置を移動）》
            evt.prevent_default();
            let script = ruby_wrap_js();
            let _ = document::eval(&script);
        }
    };

    rsx! {
        div {
            class: "editor_layout",
            // Header / Menu Bar
            header {
                class: "editor_header",
                button {
                    class: "back_button",
                    onclick: move |_| navigator.go_back(),
                    "←" 
                }
                div {
                    class: "menu_bar",
                    div { class: "{tab_class(MenuTab::File)}", onclick: move |_| selected_tab.set(MenuTab::File), "ファイル" }
                    div { class: "{tab_class(MenuTab::Edit)}", onclick: move |_| selected_tab.set(MenuTab::Edit), "編集" }
                    div { class: "{tab_class(MenuTab::Insert)}", onclick: move |_| selected_tab.set(MenuTab::Insert), "挿入" }
                    div { class: "{tab_class(MenuTab::Format)}", onclick: move |_| selected_tab.set(MenuTab::Format), "書式" }
                    div { class: "{tab_class(MenuTab::Tools)}", onclick: move |_| selected_tab.set(MenuTab::Tools), "ツール" }
                }
                div {
                    style: "margin-left: auto; color: var(--text-information); font-size: 0.9rem;",
                    "{series_title} - {chapter_title}"
                }
            }
            
            // Ribbon Toolbar
            div {
                class: "ribbon_container",
                match selected_tab() {
                    MenuTab::File => rsx! {
                        button { 
                            class: "ribbon_button", 
                            onclick: handle_save,
                            "保存" 
                        }
                        button { class: "ribbon_button", "書誌情報" }
                        button { class: "ribbon_button", "書き出し" }
                    },
                    MenuTab::Edit => rsx! {
                        button { class: "ribbon_button", "元に戻す" }
                        button { class: "ribbon_button", "やり直す" }
                    },
                    MenuTab::Insert => rsx! {
                        button { class: "ribbon_button", "ルビ" }
                        button { class: "ribbon_button", "傍点" }
                        button { class: "ribbon_button", "注記" }
                        button { class: "ribbon_button", "画像" }
                        button { class: "ribbon_button", "外字" }
                        button { class: "ribbon_button", "改ページ" }
                    },
                    MenuTab::Format => rsx! {
                        button { class: "ribbon_button", "見出し" }
                        button { class: "ribbon_button", "字下げ" }
                        button { class: "ribbon_button", "縦中横" }
                        button { class: "ribbon_button", "罫囲み" }
                    },
                    MenuTab::Tools => rsx! {
                        button { class: "ribbon_button", "縦書きプレビュー" }
                        button { class: "ribbon_button", "文字種チェック" }
                    },
                }
            }

            // Main Content Area
            main {
                class: "editor_content",
                div {
                    class: "text_area_container",
                    textarea {
                        id: "main_editor",
                        class: "main_editor",
                        placeholder: "ここに入力...",
                        spellcheck: "false",
                        value: "{content}",
                        oninput: move |evt| content.set(evt.value()),
                        onkeydown: handle_keydown,
                    }
                }
            }
        }
    }
}
