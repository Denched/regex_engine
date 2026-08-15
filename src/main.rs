use regex_engine::{compile_search, parse, run};
use std::io::{self, BufRead, Write};

fn main() {
    let pattern = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: regex_engine <pattern>");
            std::process::exit(1);
        }
    };

    let ast = match parse(&pattern) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("invalid pattern: {:?}", e);
            std::process::exit(1);
        }
    };

    let program = compile_search(&ast);

    let stdin = io::stdin();

    print!("String: ");
    io::stdout().flush().expect("err");
    for line in stdin.lock().lines() {
        let line = line.expect("read failed");
        if run(&program, &line) {
            println!("{}", line);
        } else {
            println!("Failed match");
        }

        print!("String: ");
        io::stdout().flush().expect("err");
    }
}
