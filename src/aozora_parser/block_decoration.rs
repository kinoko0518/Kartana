struct AozoraBlock {
    inside: Vec<AozoraBlock>,
    decoration: Option<BlockDecoration>,
}

enum BlockDecoration {
    Jisage(usize),
    Keigakomi,
    Yokogumi,
    Jitsume,
    Caption,
}
