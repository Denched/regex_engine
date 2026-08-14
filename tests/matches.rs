use regex_engine::{Instructions, Regex, Token, run, scanner};

#[cfg(test)]
mod tests {
    use super::*;

    fn ab_program() -> Vec<Instructions> {
        vec![
            Instructions::Char(b'a'),
            Instructions::Char(b'b'),
            Instructions::Match,
        ]
    }

    fn a_star_b_program() -> Vec<Instructions> {
        vec![
            Instructions::Split(1, 3),
            Instructions::Char(b'a'),
            Instructions::Jmp(0),
            Instructions::Char(b'b'),
            Instructions::Match,
        ]
    }
    fn search_a_plus_program() -> Vec<Instructions> {
        vec![
            Instructions::Split(3, 1),
            Instructions::Any,
            Instructions::Jmp(0),
            Instructions::Char(b'a'),
            Instructions::Split(3, 5),
            Instructions::Match,
        ]
    }
    #[test]
    fn search_a_plus_match() {
        assert!(run(&search_a_plus_program(), "bbba"));
    }
    #[test]
    fn ab_matches_ab() {
        assert!(run(&ab_program(), "ab"));
    }
    #[test]
    fn ab_rejects_ax() {
        assert!(!run(&ab_program(), "ax"));
    }
    #[test]
    fn ab_rejects_a() {
        assert!(!run(&ab_program(), "a"));
    }

    #[test]
    fn a_star_b_matches_b() {
        assert!(run(&a_star_b_program(), "b"));
    }
    #[test]
    fn a_star_b_matches_aab() {
        assert!(run(&a_star_b_program(), "aab"));
    }
    #[test]
    fn a_star_b_rejects_aa() {
        assert!(!run(&a_star_b_program(), "aa"));
    }

    #[test]
    fn scans_escaped_star() {
        assert_eq!(
            scanner(r"a\*b"),
            vec![
                Token::Literal('a'),
                Token::Literal('*'),
                Token::Literal('b'),
            ]
        );
    }
}
