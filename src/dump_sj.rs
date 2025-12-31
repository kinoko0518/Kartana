use std::fs;
use std::path::PathBuf;
use encoding_rs::SHIFT_JIS;

fn main() {
    let mut path = PathBuf::from("src/aozora_parser/parser_test_data/桜桃.txt");
    let bytes = fs::read(&path).expect("Could not find test file");
    let (cow, _, _) = SHIFT_JIS.decode(&bytes);
    println!("{}", cow);
}
