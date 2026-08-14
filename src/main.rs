use std::io;

enum Instructions {
    Char(u8),   // carries which byte
    Jmp(usize), // usize as target index
    Split(usize, usize),
    Match,
}

fn main() {
    let program = vec![
        Instructions::Char(b'a'),
        Instructions::Char(b'b'),
        Instructions::Match,
    ];
    let program2 = vec![
        Instructions::Char(b'a'),
        Instructions::Char(b'*'),
        Instructions::Char(b'b'),
        Instructions::Match,
    ];
    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Input failed");

    // PC is an index into the Vec to indicate the instruction we are currently on
    let mut pc = 0;
    let mut sp = 0;

    loop {
        match &program[pc] {
            Instructions::Char(expected) => match input.as_bytes().get(sp) {
                Some(&b) if b == *expected => {
                    sp += 1;
                    pc += 1;
                }
                _ => {
                    break;
                }
            },
            Instructions::Match => {
                println!("We got a match!");
                break;
            }
            Instructions::Jmp(target) => {
                pc = *target;
            }
            Instructions::Split(_, _) => {
                todo!()
            }
        }
    }
    // let pattern = "ab";
    // let input = "ab";

    // let (mut pc, mut sp) = (0, 0);

    // let mut buffer = String::new();

    // io::stdin().read_line(&mut buffer).expect("ee");

    // println!("test {}", buffer);

    // let testInput = input.as_bytes();
    // let patternTest = pattern.as_bytes();

    // for i in patternTest {
    //     match testInput.get(sp) {
    //         Some(&b) if b == *i => sp += 1,
    //         _ => {}
    //     }
    // }

    // println!("{}", sp);

    // match _ {
    //     Instructions::CHAR => {}
    //     Instructions::JMP => {}
    //     Instructions::SPLIT => {}
    //     Instructions::MATCH => {},
    //     _ => println!("Not implemented!")
    // }
}
