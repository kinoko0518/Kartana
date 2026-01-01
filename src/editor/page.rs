use dioxus::prelude::*;
use crate::top_page::works::ActionIcon;
use super::rich_editor::RichEditor;
use super::hooks::use_file::use_editor_file;
use super::hooks::use_linter::use_editor_linter;
use super::actions;

const BACK_ICON: Asset = asset!("/assets/icons/back.svg");

#[component]
pub fn Editor(series_title: String, chapter_title: String) -> Element {
    let navigator = use_navigator();
    
    // Custom Hooks
    // Note: use_editor_file manages loading on mount
    let mut file = use_editor_file(series_title.clone(), chapter_title.clone());
    let mut linter = use_editor_linter();

    // Event Handlers
    let mut handle_save = move |_| {
        linter.run_lint(&(file.content)());
        file.save();
    };

    let handle_change = move |new_text: String| {
        file.content.set(new_text);
    };

    // Keybinding Handler
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let modifiers = evt.modifiers();
        let key_str = key.to_string();

        if key_str == "Tab" && !modifiers.shift() && !modifiers.ctrl() && !modifiers.alt() && !modifiers.meta() {
            evt.prevent_default();
            let script = actions::script_insert_text("［＃３字下げ］");
            spawn(async move { let _ = document::eval(&script); });
        } else if key_str == "Enter" && modifiers.ctrl() {
            evt.prevent_default();
            let script = actions::script_insert_text("\n［＃改頁］");
            spawn(async move { let _ = document::eval(&script); });
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() && modifiers.shift() {
            evt.prevent_default();
            let script = actions::script_wrap_selection("［＃「", "」に傍点］", true);
            spawn(async move { let _ = document::eval(&script); });
        } else if (key_str == "<" || key_str == ",") && modifiers.ctrl() {
            evt.prevent_default();
            let script = actions::script_ruby_wrap();
            spawn(async move { let _ = document::eval(&script); });
        } else if (key_str == "s" || key_str == "S") && modifiers.ctrl() {
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
                div {
                    style: "margin-left: 10px; color: var(--text-label); font-size: 0.8rem;",
                    "{file.status}"
                }
            }

            main {
                class: "editor_content",
                div {
                    class: "text_area_container",
                    RichEditor {
                        content: file.content,
                        extra_decorations: linter.decorations,
                        onchange: handle_change,
                        onkeydown: handle_keydown,
                    }
                }
            }
        }
    }
}
