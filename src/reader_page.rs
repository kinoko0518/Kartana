use dioxus::prelude::*;
use std::fs;
use encoding_rs::SHIFT_JIS;
use crate::top_page::works::{ActionIcon, Series};

const BACK_ICON: Asset = asset!("/assets/icons/back.svg");

#[component]
pub fn Reader(series_title: String, chapter_title: String) -> Element {
    let navigator = use_navigator();
    let mut xhtml_content = use_signal(|| String::new());
    let mut author_name = use_signal(|| String::new());

    // Helper to get file path
    let file_path = {
        let s_title = series_title.clone();
        let c_title = chapter_title.clone();
        move || Series::series_dir(&s_title).join(format!("{}.txt", c_title))
    };

    use_effect(move || {
        let path = file_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let (cow, _, _) = SHIFT_JIS.decode(&bytes);
                let text = cow.into_owned();
                
                // Call text_to_xhtml which now returns XhtmlOutput struct
                match aozora_parser::text_to_xhtml(text) {
                    Ok(output) => {
                        // Inject CSS
                        let css = aozora_parser::default_css();
                        let default_style_tag = format!("<style>{}</style>", css);
                        
                        // We inject the CSS content inline to avoid path resolution issues in srcdoc iframe
                        // This assumes the assets directory is in the current working directory
                        // "include_str" is not used as requested, using runtime read.
                        let variables_css_content = fs::read_to_string("assets/css/variables.css")
                            .unwrap_or_else(|_| "/* Failed to load variables.css */".to_string());

                        let reader_css_content = fs::read_to_string("assets/css/reader.css")
                            .unwrap_or_else(|_| "/* Failed to load reader.css */".to_string());
                        
                        let variables_style_tag = format!("<style>{}</style>", variables_css_content);
                        let custom_style_tag = format!("<style>{}</style>", reader_css_content);
                        
                        let replacement = format!("{}{}{}", default_style_tag, variables_style_tag, custom_style_tag);

                        // Replace the external link with inline style + link to reader.css
                        let final_xhtml = output.xhtml.replace(
                            r#"<link rel="stylesheet" type="text/css" href="../style/book-style.css"/>"#, 
                            &replacement
                        );
                        
                        xhtml_content.set(final_xhtml);
                        author_name.set(output.metadata.author);
                    },
                    Err(_) => {
                        xhtml_content.set("Error parsing Aozora text.".to_string());
                    }
                }
            } else {
                xhtml_content.set("Error reading file.".to_string());
            }
        } else {
             xhtml_content.set("File not found.".to_string());
        }
    });

    rsx! {
        div {
            class: "reader_layout",
            
            // Header
            header {
                class: "reader_header",
                ActionIcon {
                    icon: BACK_ICON,
                    onclick: move |_| navigator.go_back(),
                }
                div {
                    class: "header_info",
                    span {
                        class: "series_info",
                        "{series_title} - {author_name}"
                    }
                    span {
                        class: "chapter_title_display",
                        "{chapter_title}"
                    }
                }
            }

            // Reader Content
            div {
                class: "reader_content",
                iframe {
                    class: "reader_iframe",
                    srcdoc: "{xhtml_content}",
                }
            }
        }
    }
}
