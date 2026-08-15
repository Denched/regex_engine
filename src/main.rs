use regex_engine::{compile_search, disassemble, parse, run, scanner};
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let debug = args.iter().any(|a| a == "--debug");
    // first string that isnt actually a flag is the pattern
    let pattern = match args.iter().find(|a| !a.starts_with("--")) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: regex_engine <pattern> [--debug]");
            std::process::exit(1);
        }
    };

    if debug {
        println!("pattern: {:?}", pattern);
        println!("\n--- tokens ---");
        println!("{:?}", scanner(&pattern));
    }

    let ast = match parse(&pattern) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("invalid pattern: {:?}", e);
            std::process::exit(1);
        }
    };

    if debug {
        println!("\n--- ast ---");
        println!("{:#?}", ast);
    }

    let program = compile_search(&ast);

    if debug {
        println!("\n--- bytecode ---");
        disassemble(&program);
        println!();
    }

    let stdin = io::stdin();
    print!("String: ");
    io::stdout().flush().expect("flush failed");

    for line in stdin.lock().lines() {
        let line = line.expect("read failed");
        if run(&program, &line) {
            println!("match");
        } else {
            println!("no match");
        }
        print!("String: ");
        io::stdout().flush().expect("flush failed");
    }
}
