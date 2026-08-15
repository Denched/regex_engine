use regex::Regex;

fn main() {
    let re = Regex::new(r"\b\w{13}\b").unwrap();

    let test = re.is_match("I categorically deny having triskaidekaphobia.");

    println!("{test}")
}
