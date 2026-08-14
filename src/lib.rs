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

// Rules (Will put in another file later)
// Chapter 4-6 of Crafting Interpreters
pub fn atom(tokens: &[Token], pos: &mut usize) -> Regex {
    match tokens[*pos] {
        Token::Literal(c) => {
            *pos += 1;
            Regex::Char(c as u8)
        }
        Token::Dot => {
            *pos += 1;
            Regex::Any
        }
        Token::Caret => {
            *pos += 1;
            Regex::Start
        }
        Token::Dollar => {
            *pos += 1;
            Regex::End
        }
        Token::OpenParen => {
            *pos += 1;
            let inner = regex(tokens, pos); // make a new "sub program" between the brackets and return it as a single atom again
            //consume close paren
            *pos += 1;
            inner
        }
        _ => panic!("custom error msg should go here prob"),
    }
}
pub fn repeat(tokens: &[Token], pos: &mut usize) -> Regex {
    let atom_re = atom(tokens, pos);
    // .get for out of bounds checks
    match tokens.get(*pos) {
        Some(Token::Star) => {
            *pos += 1;
            Regex::Star(Box::new(atom_re))
        }
        Some(Token::Plus) => {
            *pos += 1;
            Regex::Plus(Box::new(atom_re))
        }
        Some(Token::Question) => {
            *pos += 1;
            Regex::Question(Box::new(atom_re))
        }
        _ => atom_re, // no operator so return atom itself,
    }
}

// Gather all adjacent items in a direct sequence and combine them into AST
pub fn concat(tokens: &[Token], pos: &mut usize) -> Regex {
    let repeat_re = repeat(tokens, pos);

    let mut items = vec![repeat_re];

    // Keep looking ahead and add tokens to items
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Pipe | Token::CloseParen => break,
            _ => {
                items.push(repeat(tokens, pos));
            }
        }
    }

    if items.len() == 1 {
        items.pop().unwrap()
    } else {
        let mut iter = items.into_iter();
        let first = iter.next().unwrap();

        // Combines into a formal AST for elements in items
        iter.fold(first, |acc, next| {
            Regex::Concat(Box::new(acc), Box::new(next))
        })
    }
}

pub fn alternation(tokens: &[Token], pos: &mut usize) -> Regex {
    let concat_re = concat(tokens, pos);

    let mut items = vec![concat_re];
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Pipe => {
                *pos += 1;
                items.push(concat(tokens, pos));
            }
            _ => break,
        }
    }

    if items.len() == 1 {
        items.pop().unwrap()
    } else {
        let mut iter = items.into_iter();
        let first = iter.next().unwrap();

        // Combines into a formal AST for elements in items
        iter.fold(first, |acc, next| Regex::Alt(Box::new(acc), Box::new(next)))
    }
}
pub fn regex(tokens: &[Token], pos: &mut usize) -> Regex {
    alternation(tokens, pos)
}

pub fn parse(input: &str) -> Regex {
    let tokens = scanner(input);

    let mut pos = 0;
    let result = regex(&tokens, &mut pos);

    result
}

pub fn compile(re: &Regex) {}
