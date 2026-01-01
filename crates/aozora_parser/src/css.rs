
pub fn default_css() -> String {
    let mut css = String::new();

    // Import order matches book-style.css imports
    css.push_str(include_str!("epub_template/css/style-reset.css"));
    css.push_str("\n");
    css.push_str(include_str!("epub_template/css/style-standard.css"));
    css.push_str("\n");
    css.push_str(include_str!("epub_template/css/style-advance.css"));
    css.push_str("\n");
    css.push_str(include_str!("epub_template/css/aozora.css"));
    css.push_str("\n");
    css.push_str(include_str!("epub_template/css/font.css"));
    css.push_str("\n");
    css.push_str(include_str!("epub_template/css/text.css"));
    css.push_str("\n");
    
    // book-style.css contains customizations. We should include it but remove the @imports
    // because we just inlined them. 
    // However, for simplicity, we can include the whole file. 
    // Browsers ignore @import if it's not at the start (which it won't be since we pushed other stuff before).
    // Or if it refers to a file that doesn't exist, it will just fail to import that bit, which is fine since we inlined it.
    css.push_str(include_str!("epub_template/css/book-style.css"));
    
    css
}
