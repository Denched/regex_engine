//! Regex Engine
//! Utilises Pike VM architecture
//! Runs each step from scanner -> parsing -> compiling(bytecode) -> execute to calculate the regex expression and pattern match

pub mod rules;

pub use rules::parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instructions {
    Char(u8),   // carries which byte
    Jmp(usize), // usize as target index
    Split(usize, usize),
    Match,
    Any,
    Caret,
    Dollar,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken(usize), // position in token stream
    UnexpectedEnd,
    UnmatchedParen(usize),
    TrailingTokens(usize),
    DanglingOperator(usize), // e.g. "*ab" — operator with no atom before it
}

/// AST representation of a regex expression
#[derive(Debug, PartialEq, Eq)]
pub enum Regex {
    Char(u8),
    Any,
    Concat(Box<Regex>, Box<Regex>),
    Alt(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>),
    Question(Box<Regex>),
    Start,
    End,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Literal(char),
    Dot,        // '.'
    Star,       // '*'
    Plus,       // '+'
    Question,   // '?'
    Pipe,       // '|'
    OpenParen,  // '('
    CloseParen, // ')'
    Caret,      // '^'
    Dollar,     // '$'
}
// Recursive function utilized to lazily add threads to the list and sets the base case as either CHAR or MATCH
fn add_thread(
    list: &mut Vec<usize>,
    visited: &mut Vec<usize>,
    pc: usize,
    program: &[Instructions],
    sp: usize,
    input_len: usize,
) {
    // Prevent inf loops so we dont keep visiting the same instruction twice in one step
    if list.contains(&pc) || visited.contains(&pc) {
        return;
    }
    visited.push(pc);
    match program[pc] {
        Instructions::Char(_) | Instructions::Match | Instructions::Any => {
            list.push(pc);
        } //base case
        Instructions::Jmp(target) => {
            add_thread(list, visited, target, program, sp, input_len);
        }
        Instructions::Split(t1, t2) => {
            add_thread(list, visited, t1, program, sp, input_len);
            add_thread(list, visited, t2, program, sp, input_len);
        }
        Instructions::Caret => {
            if sp == 0 {
                add_thread(list, visited, pc + 1, program, sp, input_len);
            }
        }
        Instructions::Dollar => {
            if sp == input_len {
                add_thread(list, visited, pc + 1, program, sp, input_len);
            }
        }
    }
}

pub fn run(program: &[Instructions], input: &str) -> bool {
    let mut c_list: Vec<usize> = vec![]; // threads for curr char
    let mut n_list: Vec<usize> = vec![]; // threads being built for next char

    {
        let mut visited: Vec<usize> = vec![];

        // Add initial pc(s) in c_list at the very start of the program to process later
        add_thread(&mut c_list, &mut visited, 0, program, 0, input.len());
    }
    // sp = current pos in the string
    for sp in 0..=input.len() {
        let mut visited: Vec<usize> = vec![];

        // Check if atleast one active thread in the execution list has reached MATCH
        if c_list
            .iter()
            .any(|&pc| matches!(program[pc], Instructions::Match))
        {
            return true;
        }
        // PC is an index into the Vec to indicate the instruction we are currently on

        for &pc in &c_list {
            // if let Instructions::Char(expected) = program[pc]
            //     && input.as_bytes().get(sp) == Some(&expected)
            // {
            //     add_thread(&mut n_list, &mut visited, pc + 1, program);
            // }
            // if let Instructions::Any = program[pc]
            //     && input.as_bytes().get(sp).is_some()
            // {
            //     add_thread(&mut n_list, &mut visited, pc + 1, program);
            // }

            match program[pc] {
                Instructions::Char(c) => {
                    if input.as_bytes().get(sp) == Some(&c) {
                        add_thread(
                            &mut n_list,
                            &mut visited,
                            pc + 1,
                            program,
                            sp + 1,
                            input.len(),
                        );
                    }
                }

                Instructions::Any => {
                    if input.as_bytes().get(sp).is_some() {
                        add_thread(
                            &mut n_list,
                            &mut visited,
                            pc + 1,
                            program,
                            sp + 1,
                            input.len(),
                        );
                    }
                }
                Instructions::Split(_, _) => {
                    unreachable!() // based on add_thread split and jmp should never be reachable since they are expanded away before anything lands in the list
                }
                Instructions::Jmp(_) => {
                    unreachable!()
                }
                Instructions::Caret => {
                    unreachable!()
                }
                Instructions::Dollar => {
                    unreachable!()
                }
                Instructions::Match => {} // match already handled on first if check in the loop
            }
        }
        // Iteration done, update c_list, empty out n_list to prepare for the next iteration.
        c_list = n_list;
        n_list = vec![];

        if c_list.is_empty() {
            return false;
        }
    }

    false
}

// Use raw chars to form tokens for parsing later
pub fn scanner(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        let token = match c {
            '.' => Token::Dot,
            '*' => Token::Star,
            '+' => Token::Plus,
            '?' => Token::Question,
            '|' => Token::Pipe,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '^' => Token::Caret,
            '$' => Token::Dollar,
            '\\' => {
                if let Some(escaped) = chars.next() {
                    Token::Literal(escaped)
                } else {
                    Token::Literal('\\')
                }
            }
            literal => Token::Literal(literal),
        };
        tokens.push(token);
    }
    tokens
}

// Shift so the positions still mean the same thing after being relocated
pub fn shift(program: Vec<Instructions>, offset: usize) -> Vec<Instructions> {
    program
        .into_iter()
        .map(|inst| match inst {
            Instructions::Jmp(target) => Instructions::Jmp(target + offset),
            Instructions::Split(t1, t2) => Instructions::Split(t1 + offset, t2 + offset),
            other => other,
        })
        .collect()
}
// recursive, every node gets its own independent bytecode and parent nodes stitch them together as we unwind up back the AST tree
pub fn compile(re: &Regex) -> Vec<Instructions> {
    match re {
        Regex::Char(c) => vec![Instructions::Char(*c)], //leaf nodes
        Regex::Any => vec![Instructions::Any],
        Regex::Start => vec![Instructions::Caret],
        Regex::End => vec![Instructions::Dollar],
        Regex::Concat(a, b) => {
            let compile_a = compile(a); // recurse down left tree
            let compile_b = compile(b); // right tree
            let a_len = compile_a.len();
            let mut program = compile_a;

            program.extend(shift(compile_b, a_len)); // combine them together
            program
        }

        Regex::Alt(a, b) => {
            let compile_a = compile(a); // recurse down left tree
            let compile_b = compile(b); // right tree

            let a_start = 1;
            let jmp_pos = a_start + compile_a.len();
            let b_start = jmp_pos + 1;
            let end = b_start + compile_b.len();
            // Reference
            //             Index                           Instruction
            // ─────────────────────────────────────────────────────────────────────────────
            // 0                               Split(a_start, b_start)  ◄── Fork to 'a' or 'b'
            // a_start (1)                     ┌──────────────────────┐
            // ...                             │   Bytecode for 'a'   │
            // jmp_pos (1 + len(a))            └──────────────────────┘
            //                                 Jmp(end)                 ◄── Skip 'b' if 'a' matched
            // b_start (jmp_pos + 1)           ┌──────────────────────┐
            // ...                             │   Bytecode for 'b'   │
            // end     (b_start + len(b))      └──────────────────────┘
            //                                 <Next Instruction>       ◄── Both paths exit here
            let mut program = vec![Instructions::Split(a_start, b_start)];

            program.extend(shift(compile_a, a_start));
            program.push(Instructions::Jmp(end));
            program.extend(shift(compile_b, b_start));
            program
        }
        Regex::Star(inner) => {
            let compile_inner = compile(inner);
            let inner_len = compile_inner.len();
            let mut result = Vec::new();

            // inner index starts at 1 due to split, 1 + inner_len + 1 goes to the next pattern past jmp
            result.push(Instructions::Split(1, 1 + inner_len + 1));
            result.extend(shift(compile_inner, 1));
            result.push(Instructions::Jmp(0)); // Jump back to split
            result
        }
        Regex::Plus(inner) => {
            let compile_inner = compile(inner);
            let inner_len = compile_inner.len();

            let mut result = Vec::new();

            result.extend(compile_inner);
            result.push(Instructions::Split(0, inner_len + 1));

            result
        }
        Regex::Question(inner) => {
            let compile_inner = compile(inner);
            let inner_len = compile_inner.len();

            let mut result = Vec::new();

            result.push(Instructions::Split(1, inner_len + 1));
            result.extend(shift(compile_inner, 1));

            result
        }
    }
}

/// Anchored search (Only searches for the pattern at the start of the string)
pub fn compile_program(re: &Regex) -> Vec<Instructions> {
    let mut program = compile(re);
    program.push(Instructions::Match);
    program
}
/// Unanchored search (Searches for a pattern at any part of the string)
pub fn compile_search(re: &Regex) -> Vec<Instructions> {
    let compiled = compile(re);

    // Start with .*?, search anywhere in the list unanchored
    let prefix = vec![
        Instructions::Split(3, 1),
        Instructions::Any,
        Instructions::Jmp(0),
    ];

    let prefix_len = prefix.len();

    let mut program = prefix;

    program.extend(shift(compiled, prefix_len));
    program.push(Instructions::Match);
    program
}

/// Evaluates whether a regular expression matches any part of the given input string.
///
///
///
/// # Examples
///
/// ```
/// use regex_engine::is_match;
///
/// // Success!
/// assert_eq!(is_match("a|b*c", "ac").unwrap(), true);
/// assert_eq!(is_match("a|b*c", "bbbc").unwrap(), true);
///
/// // Fail!
/// assert_eq!(is_match("a|b*c", "z").unwrap(), false);
///
/// // Invalid!
/// assert!(is_match("*abc", "abc").is_err());
/// ```
pub fn is_match(pattern: &str, input: &str) -> Result<bool, ParseError> {
    let ast = parse(pattern)?;
    let program = compile_search(&ast);
    Ok(run(&program, input))
}

// pub fn is_match_at_start(pattern: &str, input: &str) -> Result<bool, ParseError> {
//     let ast = parse(pattern)?;
//     let program = compile_program(&ast);
//     Ok(run(&program, input))
// }
