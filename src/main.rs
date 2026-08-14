use regex_engine::{Instructions, run};
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input failed");
    let input = input.trim_end();

    let program = vec![
        Instructions::Caret,
        Instructions::Char(b'a'),
        Instructions::Char(b'b'),
        Instructions::Char(b'c'),
        Instructions::Dollar,
        Instructions::Match,
    ];

    if run(&program, input) {
        println!("match");
    } else {
        println!("no match");
    }
}
