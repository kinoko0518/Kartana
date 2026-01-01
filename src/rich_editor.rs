//! Rich Editor component with decoration support.
//!
//! This module provides a `RichEditor` component that supports syntax highlighting
//! and lint decorations, similar to VSCode's text editor.

use dioxus::prelude::*;

use crate::decoration::{
    decorations_from_tokens, merge_decorations, render_to_html, split_into_segments, Decoration,
};

/// Props for the RichEditor component.
#[derive(Props, Clone, PartialEq)]
pub struct RichEditorProps {
    /// The text content
    pub content: Signal<String>,
    /// Additional decorations (e.g., from linter)
    #[props(default)]
    pub extra_decorations: ReadOnlySignal<Vec<Decoration>>,
    /// Callback when content changes
    pub onchange: EventHandler<String>,
    /// Callback for keydown events
    #[props(default)]
    pub onkeydown: EventHandler<KeyboardEvent>,
}

/// A rich text editor with decoration support.
#[component]
pub fn RichEditor(props: RichEditorProps) -> Element {
    let content = props.content;
    let extra_decorations = props.extra_decorations;

    // Compute decorations from tokens
    let decorated_html = use_memo(move || {
        let text = content();
        if text.is_empty() {
            return String::new();
        }

        // Tokenize the text
        let tokens = match aozora_parser::parse_aozora(text.clone()) {
            Ok(tokens) => tokens,
            Err(_) => return html_escape(&text),
        };

        // Get syntax decorations
        let mut decorations = decorations_from_tokens(&tokens);

        // Add extra decorations (lint warnings, etc.)
        decorations.extend(extra_decorations().iter().cloned());

        // Merge and sort
        let decorations = merge_decorations(decorations);

        // Split into segments and render
        let segments = split_into_segments(&text, &decorations);
        render_to_html(&segments)
    });

    // Handle input events from contenteditable
    let handle_input = move |_evt: Event<FormData>| {
        // Get text from the contenteditable div via JS
        let script = r#"
            (function() {
                const editor = document.getElementById('rich_editor');
                if (editor) {
                    return editor.innerText || '';
                }
                return '';
            })()
        "#;
        spawn(async move {
            if let Ok(result) = document::eval(script).await {
                if let Some(text) = result.as_str() {
                    props.onchange.call(text.to_string());
                }
            }
        });
    };

    // Sync content to the editor when it changes externally
    use_effect(move || {
        let html = decorated_html();
        // Use JS to update the editor content while preserving cursor
        let script = format!(
            r#"
            (function() {{
                const editor = document.getElementById('rich_editor');
                if (!editor) return;

                // 1. Save cursor position (plain text index)
                const selection = window.getSelection();
                let cursorIndex = 0;
                let hasFocus = document.activeElement === editor;
                
                if (hasFocus && selection.rangeCount > 0 && editor.contains(selection.anchorNode)) {{
                    const range = selection.getRangeAt(0);
                    const preCaretRange = range.cloneRange();
                    preCaretRange.selectNodeContents(editor);
                    preCaretRange.setEnd(range.endContainer, range.endOffset);
                    cursorIndex = preCaretRange.toString().length;
                }}

                // 2. Update content
                editor.innerHTML = `{}`;

                // 3. Restore cursor
                if (hasFocus) {{
                    let charCount = 0;
                    const nodeStack = [editor];
                    let found = false;

                    while (nodeStack.length > 0) {{
                        const node = nodeStack.pop();
                        if (node.nodeType === 3) {{ // Text node
                            const nextCharCount = charCount + node.length;
                            if (!found && nextCharCount >= cursorIndex) {{
                                const range = document.createRange();
                                range.setStart(node, cursorIndex - charCount);
                                range.collapse(true);
                                selection.removeAllRanges();
                                selection.addRange(range);
                                found = true;
                                break; 
                            }}
                            charCount = nextCharCount;
                        }} else {{
                            let i = node.childNodes.length;
                            while (i--) {{
                                nodeStack.push(node.childNodes[i]);
                            }}
                        }}
                    }}
                }}
            }})()
        "#,
            html.replace('`', "\\`").replace("${", "\\${")
        );
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    });

    rsx! {
        div {
            class: "rich_editor_container",
            div {
                id: "rich_editor",
                class: "rich_editor",
                contenteditable: "true",
                spellcheck: "false",
                oninput: handle_input,
                onkeydown: move |evt| props.onkeydown.call(evt),
            }
        }
    }
}

/// Escape HTML entities.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>")
}
