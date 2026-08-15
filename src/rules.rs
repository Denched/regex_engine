use crate::{ParseError, Regex, Token, scanner};

// Rules
// Chapter 4-6 of Crafting Interpreters
pub fn atom(tokens: &[Token], pos: &mut usize) -> Result<Regex, ParseError> {
    let token = tokens.get(*pos).ok_or(ParseError::UnexpectedEnd)?;

    match token {
        Token::Literal(c) => {
            *pos += 1;
            Ok(Regex::Char(*c as u8))
        }
        Token::Dot => {
            *pos += 1;
            Ok(Regex::Any)
        }
        Token::Caret => {
            *pos += 1;
            Ok(Regex::Start)
        }
        Token::Dollar => {
            *pos += 1;
            Ok(Regex::End)
        }
        Token::OpenParen => {
            let open_pos = *pos;

            *pos += 1;
            let inner = regex(tokens, pos)?; // make a new "sub program" between the brackets and return it as a single atom again
            match tokens.get(*pos) {
                Some(Token::CloseParen) => {
                    *pos += 1;
                    Ok(inner)
                }
                _ => Err(ParseError::UnmatchedParen(open_pos)),
            }
        }
        // operator with no atom beforehand
        Token::Star | Token::Plus | Token::Question => Err(ParseError::DanglingOperator(*pos)),
        _ => Err(ParseError::UnexpectedToken(*pos)),
    }
}
pub fn repeat(tokens: &[Token], pos: &mut usize) -> Result<Regex, ParseError> {
    let atom_re = atom(tokens, pos)?;
    // .get for out of bounds checks
    match tokens.get(*pos) {
        Some(Token::Star) => {
            *pos += 1;
            Ok(Regex::Star(Box::new(atom_re)))
        }
        Some(Token::Plus) => {
            *pos += 1;
            Ok(Regex::Plus(Box::new(atom_re)))
        }
        Some(Token::Question) => {
            *pos += 1;
            Ok(Regex::Question(Box::new(atom_re)))
        }
        _ => Ok(atom_re), // no operator so return atom itself,
    }
}

// Gather all adjacent items in a direct sequence and combine them into AST
pub fn concat(tokens: &[Token], pos: &mut usize) -> Result<Regex, ParseError> {
    let repeat_re = repeat(tokens, pos)?;

    let mut items = vec![repeat_re];

    // Keep looking ahead and add tokens to items
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Pipe | Token::CloseParen => break,
            _ => {
                items.push(repeat(tokens, pos)?);
            }
        }
    }

    if items.len() == 1 {
        Ok(items.pop().unwrap())
    } else {
        let mut iter = items.into_iter();
        let first = iter.next().unwrap();

        // Combines into a formal AST for elements in items
        Ok(iter.fold(first, |acc, next| {
            Regex::Concat(Box::new(acc), Box::new(next))
        }))
    }
}

pub fn alternation(tokens: &[Token], pos: &mut usize) -> Result<Regex, ParseError> {
    let concat_re = concat(tokens, pos)?;

    let mut items = vec![concat_re];
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Pipe => {
                *pos += 1;
                items.push(concat(tokens, pos)?);
            }
            _ => break,
        }
    }

    if items.len() == 1 {
        Ok(items.pop().unwrap())
    } else {
        let mut iter = items.into_iter();
        let first = iter.next().unwrap();

        // Combines into a formal AST for elements in items
        Ok(iter.fold(first, |acc, next| Regex::Alt(Box::new(acc), Box::new(next))))
    }
}
pub fn regex(tokens: &[Token], pos: &mut usize) -> Result<Regex, ParseError> {
    alternation(tokens, pos)
}

pub fn parse(input: &str) -> Result<Regex, ParseError> {
    let tokens = scanner(input);

    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEnd);
    }

    let mut pos = 0;
    let result = regex(&tokens, &mut pos)?;

    // e.g "a)"
    if pos != tokens.len() {
        return Err(ParseError::TrailingTokens(pos));
    }
    Ok(result)
}
