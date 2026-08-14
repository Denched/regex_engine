#[derive(Debug, Clone, Copy)]
pub enum Instructions {
    Char(u8),   // carries which byte
    Jmp(usize), // usize as target index
    Split(usize, usize),
    Match,
    Any,
    Caret,
    Dollar,
}

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
/// Recursive function utilized to lazily add threads to the list and sets the base case as either CHAR or MATCH
///
/// e.g
/// add_thread(&mut c_list, &mut visited, 0, &program);
/// add_thread(&mut n_list, &mut visited, pc + 1, &program);
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

pub fn parse(input: &str) {
    let tokens = scanner(input);
}

pub fn compile(re: &Regex) {}
