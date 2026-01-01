use dioxus::prelude::*;
use crate::decoration::{decorations_from_lint, Decoration};

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditorLinter {
    pub decorations: Signal<Vec<Decoration>>,
}

impl UseEditorLinter {
    pub fn run_lint(&mut self, text: &str) {
        if text.is_empty() {
            self.decorations.set(Vec::new());
            return;
        }

        if let Ok(tokens) = aozora_parser::parse_aozora(text.to_string()) {
            if let Ok(doc) = aozora_parser::parse(tokens) {
                if let Ok(block) = aozora_parser::parse_blocks(doc.items) {
                    let result = aozora_parser::lint(block, text);
                    let new_decorations = decorations_from_lint(&result.warnings);
                    self.decorations.set(new_decorations);
                }
            }
        }
    }
}

pub fn use_editor_linter() -> UseEditorLinter {
    let decorations = use_signal(|| Vec::<Decoration>::new());
    
    UseEditorLinter {
        decorations,
    }
}
