#[derive(Debug, Clone, Copy)]
pub enum Instructions {
    Char(u8),   // carries which byte
    Jmp(usize), // usize as target index
    Split(usize, usize),
    Match,
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
) {
    // Prevent inf loops so we dont keep visiting the same instruction twice in one step
    if list.contains(&pc) || visited.contains(&pc) {
        return;
    }
    visited.push(pc);
    match program[pc] {
        Instructions::Char(_) | Instructions::Match => {
            list.push(pc);
        } //base case
        Instructions::Jmp(target) => {
            add_thread(list, visited, target, program);
        }
        Instructions::Split(t1, t2) => {
            add_thread(list, visited, t1, program);
            add_thread(list, visited, t2, program);
        }
    }
}

pub fn run(program: &[Instructions], input: &str) -> bool {
    let mut c_list: Vec<usize> = vec![]; // threads for curr char
    let mut n_list: Vec<usize> = vec![]; // threads being built for next char

    {
        let mut visited: Vec<usize> = vec![];

        // Add initial pc(s) in c_list at the very start of the program to process later
        add_thread(&mut c_list, &mut visited, 0, program);
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
            if let Instructions::Char(expected) = program[pc]
                && input.as_bytes().get(sp) == Some(&expected)
            {
                add_thread(&mut n_list, &mut visited, pc + 1, program);
            }
        }
        // Iteration done, update c_list, empty out n_list to prepare for the next iteration.
        c_list = n_list;
        n_list = vec![];

        if c_list.is_empty() {
            println!("No match found");
            return false;
        }
    }

    false
}
