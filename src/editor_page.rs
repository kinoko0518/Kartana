use dioxus::prelude::*;

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

    let tab_class = |tab: MenuTab| {
        if selected_tab() == tab {
            "menu_item active"
        } else {
            "menu_item"
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
                        button { class: "ribbon_button", "保存" }
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
                        class: "main_editor",
                        placeholder: "ここに入力...",
                        spellcheck: "false",
                    }
                }
            }
        }
    }
}
