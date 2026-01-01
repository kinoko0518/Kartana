//! Helper functions to generate JavaScript for editor interactions

/// Generate JS to insert text at cursor position (Undo-compatible)
pub fn script_insert_text(text: &str) -> String {
    format!(r#"
        const editor = document.getElementById('rich_editor');
        if (editor) {{
            editor.focus();
            document.execCommand('insertText', false, "{}");
        }}
    "#, text.replace("\"", "\\\"").replace("\n", "\\n"))
}

/// Generate JS to wrap selection with prefix/suffix
pub fn script_wrap_selection(prefix: &str, suffix: &str, keep_original: bool) -> String {
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
}

/// Generate JS for ruby wrapping (《》) and cursor placement
pub fn script_ruby_wrap() -> String {
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
}
