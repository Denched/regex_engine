use regex_engine::*;

#[cfg(test)]
mod compiler_tests {
    use super::*;

    // ---------- shift ----------

    #[test]
    fn shift_char_unchanged() {
        let prog = vec![
            Instructions::Char(b'a'),
            Instructions::Any,
            Instructions::Match,
        ];
        let shifted = shift(prog.clone(), 5);
        assert_eq!(shifted, prog); // no targets to move, should be identical
    }

    #[test]
    fn compiles_single_char() {
        let re = Regex::Char(b'a');
        assert_eq!(compile(&re), vec![Instructions::Char(b'a')]);
    }

    #[test]
    fn compiles_any() {
        let re = Regex::Any;
        assert_eq!(compile(&re), vec![Instructions::Any]);
    }

    #[test]
    fn compiles_start_and_end() {
        assert_eq!(compile(&Regex::Start), vec![Instructions::Caret]);
        assert_eq!(compile(&Regex::End), vec![Instructions::Dollar]);
    }

    #[test]
    fn compiles_concat_ab() {
        let re = Regex::Concat(Box::new(Regex::Char(b'a')), Box::new(Regex::Char(b'b')));
        assert_eq!(
            compile(&re),
            vec![Instructions::Char(b'a'), Instructions::Char(b'b')]
        );
    }

    #[test]
    fn compiles_alt_ab() {
        let re = Regex::Alt(Box::new(Regex::Char(b'a')), Box::new(Regex::Char(b'b')));
        assert_eq!(
            compile(&re),
            vec![
                Instructions::Split(1, 3),
                Instructions::Char(b'a'),
                Instructions::Jmp(4),
                Instructions::Char(b'b'),
            ]
        );
    }

    #[test]
    fn compiles_star_a() {
        let re = Regex::Star(Box::new(Regex::Char(b'a')));
        assert_eq!(
            compile(&re),
            vec![
                Instructions::Split(1, 3),
                Instructions::Char(b'a'),
                Instructions::Jmp(0),
            ]
        );
    }

    #[test]
    fn compiles_star_shifts_inner() {
        // (a|b)*
        let re = Regex::Star(Box::new(Regex::Alt(
            Box::new(Regex::Char(b'a')),
            Box::new(Regex::Char(b'b')),
        )));
        assert_eq!(
            compile(&re),
            vec![
                Instructions::Split(1, 6), // enter alt, or skip past whole block
                Instructions::Split(2, 4), // inner alt's split, shifted by 1
                Instructions::Char(b'a'),
                Instructions::Jmp(5), // shifted by 1
                Instructions::Char(b'b'),
                Instructions::Jmp(0), // star's own loop-back
            ]
        );
    }

    #[test]
    fn compiles_plus_a() {
        let re = Regex::Plus(Box::new(Regex::Char(b'a')));
        assert_eq!(
            compile(&re),
            vec![Instructions::Char(b'a'), Instructions::Split(0, 2)]
        );
    }

    #[test]
    fn compiles_question_a() {
        let re = Regex::Question(Box::new(Regex::Char(b'a')));
        assert_eq!(
            compile(&re),
            vec![Instructions::Split(1, 2), Instructions::Char(b'a')]
        );
    }

    #[test]
    fn compiles_alt_shifts_inner() {
        let re = Regex::Question(Box::new(Regex::Alt(
            Box::new(Regex::Char(b'a')),
            Box::new(Regex::Char(b'b')),
        )));
        assert_eq!(
            compile(&re),
            vec![
                Instructions::Split(1, 5),
                Instructions::Split(2, 4),
                Instructions::Char(b'a'),
                Instructions::Jmp(5),
                Instructions::Char(b'b'),
            ]
        );
    }

    fn matches(pattern: &str, input: &str) -> bool {
        let ast = parse(pattern).expect("should parse");
        let program = compile_search(&ast);
        run(&program, input)
    }

    #[test]
    fn pipeline_literal_concat() {
        assert!(matches("ab", "ab"));
        assert!(!matches("ab", "ax"));
    }

    #[test]
    fn pipeline_star() {
        assert!(matches("a*b", "aab"));
        assert!(matches("a*b", "b"));
        assert!(!matches("a*b", "aa"));
    }
}
