use regex::Regex;

#[test]
fn oracle_sanity() {
    let re = Regex::new(r"\b\w{13}\b").unwrap();
    assert!(re.is_match("I categorically deny having triskaidekaphobia."));
}

fn main() {
    let re = Regex::new(r"\b\w{13}\b").unwrap();

    let test = re.is_match("I categorically deny having triskaidekaphobia.");

    println!("{test}")
}
