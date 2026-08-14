use regex_engine::{Instructions, run};
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input failed");
    let input = input.trim_end();

    let program = vec![
        Instructions::Split(1, 3),
        Instructions::Char(b'a'),
        Instructions::Jmp(0),
        Instructions::Char(b'b'),
        Instructions::Match,
    ];

    if run(&program, input) {
        println!("match");
    } else {
        println!("no match");
    }
}
