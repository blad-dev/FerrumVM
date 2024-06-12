use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let reader = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
    for line in reader.lines() {
        for word in line.unwrap().split_whitespace() {
            println!("word '{}'", word);
        }
    }
}
