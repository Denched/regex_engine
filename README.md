<div align="center">
  <h1><b>regex_engine</b></h1>
  <p>A regular expression engine in Rust...</p>
  <img src="https://github.com/Denched/regex_engine/actions/workflows/rust.yml/badge.svg" alt="CI" />
</div>

<!-- <div align="center">
  <img src="./assets/demo.gif" width="800" alt="regex_engine CLI demo" />
</div> -->
---

## So why does this exist?

Most regular expression engines (Perl, Python, Java, JavaScript, etc.) match using recursive backtracking. This is simple to implement. Although, results in an exponential worst-case runtime.

My engine uses Thompson's NFA(Non-Deterministic Finite Automata) Simulation. It is the opposite of recursive backtracking. Rather than picking a branch and rewinding, it advances every single branch, and simultaneously, all branches in lockstep. Thompson's NFA Simulation also tracks a set of reachable states rather than tracking paths, and one pass over the input results in a runtime of O(n·m), making it much more efficient.

### Demonstration

The pattern (a+)+b matches a string of a’s with no b’s, and is a classic example of catastrophic backtracking. Every a in the string doubles the number of paths a backtracking engine has to verify to determine that there is no match.


| n | Python `re` | `regex_engine` | `regex` crate |
|---:|---:|---:|---:|
| 10 | 0.18 ms | 2,370 ns | 34.8 ns |
| 12 | 0.37 ms | 2,770 ns | 37.2 ns |
| 14 | 1.30 ms | 3,250 ns | 41.2 ns |
| 16 | 5.21 ms | 3,690 ns | 45.2 ns |
| 18 | 21.3 ms | 4,140 ns | 46.4 ns |
| 20 | 71.1 ms | 4,580 ns | 51.2 ns |
| 22 | 270 ms | 5,050 ns | 52.6 ns |
| 24 | 1.16 s | 5,760 ns | 56.2 ns |
| 26 | 4.33 s | 5,950 ns | 59.3 ns |
| 28 | 17.5 s | 6,350 ns | 60.5 ns |
| 30 | 71.6 s | 6,790 ns | 63.8 ns |


Python's time increases roughly by a factor of 4 each time two characters are added. The engine's time increases linearly from 2.2μs to 6.8μs for the same input. For large values, about n=40 would take around 20 hours, and n=50 would take about 200 years to complete.

The result is from the median of 4 runs. Rust’s times were taken from cargo bench (criterion, with 100 samples each); for Python 3, times were captured with time.perf_counter(). Both were used on the same machine, and compiled releases were used.

### Realistic patterns

| Pattern | Input | `regex_engine` | `regex` crate | Ratio |
|---|---|---:|---:|---:|
| `^abc$` | `abc` | 778 ns | 23.5 ns | 33× |
| `hello` | `hello world hello` | 1,180 ns | 14.4 ns | 82× |
| `a*b` | `aaaaaaaaaab` | 2,660 ns | 38.4 ns | 69× |
| `(cat\|dog\|bird)` | `I have a bird at home` | 4,570 ns | 47.6 ns | 96× |

### Against the production crate

This engine is at least 65-100 times slower than Rust's `regex` crate. Both crates use the
same class of algorithm.

Reason:

- Cache lazy Thompson's simulation. Thompson simulation rebuilds the same sets of
  states on every pass. Caching sets most of the transitions to a single memory
  lookup.
- Perform literal substring filtering. All production regex engines perform a SIMD
  scan for the required literal substring before running the regex
- Use `Vec::contains()` to deduplicate threads, which is O(n) for each
  check. This can be O(1) using a bitset or generation stamps.
- Use a fresh `Vec` allocation for each step instead of swapping two
  preallocated buffers.

None of the above actually changes complexity. These are all constant-factor work related
implements, as evidenced by the numbers.

Anchored inputs are the fastest case. `^` fails on position 0 to check for any
non matching input, thus the `.*?` search prefix fails and threads die.

Reproduce with `cargo bench` and `python benches/compare.py`.


## Getting Started

### Use as a CLI

```bash
git clone https://github.com/Denched/regex_engine.git
cd regex_engine
cargo build --release


cargo run --release -- "a*b"

cargo run --release -- "a*b" --debug
```
### Use as a library

```rust
use regex_engine::is_match;

fn main() {
    // Searches anywhere in the input by default
    assert_eq!(is_match("bc", "abcd"), Ok(true));

    // Use ^ and $ to anchor, start and end respectively
    assert_eq!(is_match("^bc", "abcd"), Ok(false));

    // Distorted patterns return errors
    assert!(is_match("a(b", "abc").is_err());
}
```

---

## Supported syntax

| Syntax | Meaning |
|---|---|
| `abc` | Literal characters, concatenation |
| `a\|b` | Alternation |
| `a*` | Zero or more |
| `a+` | One or more |
| `a?` | Zero or one |
| `(ab)*` | Grouping |
| `.` | Any single byte |
| `^` `$` | Start / end of input anchors |
| `\*` `\|` etc. | Escaped metacharacters |

Not supported, purposely: backreferences (\1), due to their NP-complete nature and the incompatibility with the approach of linear time since they mean a thread's capture state impacts future matchings and identical states can no longer be combined.

---
## How it works

The pattern goes through four stages:

```
"a*b"  ──scanner──▶  [Literal('a'), Star, Literal('b')]
       ──parser───▶  Concat(Star(Char('a')), Char('b'))
       ──compiler─▶  [Split(1,3), Char('a'), Jmp(0), Char('b'), Match]
       ──VM───────▶  true / false
```

**1. Scanner** -- pattern strings to a flat token stream. Handles escapes.

**2. Parser** -- performs a recursive decent over the grammar below and builds an AST. Precedence is weakest to strongest of alternation, concatenation, and the repetition operators.

```
regex       := alternation
alternation := concat ( '|' concat )*
concat      := repeat ( repeat )*
repeat      := atom ( '*' | '+' | '?' )?
atom        := CHAR | '.' | '^' | '$' | '(' regex ')'
```

Concatenation is the tricky rule. Since there is no operator character, the loop will continue if the next token could start an atom as opposed to looking for a separator.

**3. Compiler** -- parses ASTs to bytecodes in a recursive fashion. Each subtree will compile to a self contained instruction sequence with a relative offset to its starting point, and then the offsets will be adjusted by parent nodes for instruction combination.

**4. VM** -- runs the bytecodes with input provided.


### Instruction set

Four primary definitions and three extensions:

| Definition | Behavior |

|----|----|

| `Char`(b) | Consume one byte if it equals b, otherwise terminate the thread |

| `Any` | Consume any one byte|

| `Match` | This thread has completed its task |

| `Jmp`(x) | Set PC to x and consume nothing |

| `Split`(x, y) | Continue execution at both x and y - consumes nothing |

| `Caret` | Ensure that the position is set to 0 - consumes nothing |

| `Dollar` | Ensure that the position is set to end of input - consumes nothing |

`Char` and `Any` are the only instructions that consume input. `Match`, and all others, consume nothing and are therefore, epsilon transitions in NFA.

`Split` is the main construction. It does not `choose` a branch. It forks, and depending on the input, one of the branches dies.

### Why it's linear

VMs have two lists of thread positions; one for the current character, and one for the next. In VMs all threads move in lockstep, so they share a string position, making a thread essentially a program counter. Therefore, the number of distinct threads is the same as the number of instructions.

Since each entry in the lists is a program counter, we can deduplicate entries to cap each list at the program length. This makes character-level work `O(m)`, and overall work `O(n·m)`, where `n` is the number of backtrackings a naive engine would make.

---

## Design decisions

**Bytecode VM vs direct NFA graph simulation.** One advantage of VM over NFA graph simulation is that the front end (scanner, parser, compiler) is shared and the execution strategy is swappable. With a flat instruction array, the intermediate representation can be printed and debugged, which is difficult to do with a graph of pointers.

**Bytes (u8) vs Unicode scalars (char).** Indexing a byte slice is O(1) whereas indexing a &str by character position is O(n) and therefore not possible in Rust. Since UTF-8 is self-synchronizing, multiple-byte characters compile to consecutive Char instructions. The tradeoff is that . does not match one character but rather one byte, so the engine is ASCII-centric for the any-character metacharacter.

**Index shifting over true backpatching.** Because the AST is built before compilation, the size of each subtree is known before a parent needs to refer to it, so each compiled fragment is fully resolved. Compiling in linear passes wouldn't provide a tree for Cox to lean on and therefore does require backpatching.

**Search by default, anchor by opt-in.** The alternative is to consider every possible starting position and match each starting position, which is O(n²) and is therefore unacceptable. The compiler prepends the bytecode for .*? to every program, and the VM then explores all starting positions in a linear pass. In this case, the search prefix is correct since ^ and $ are gate position instructions.

**Perl (leftmost-first) semantics rather than POSIX (leftmost-longest).** Falls out of `Split` preferring its first argument, which is also what makes non-greedy operators expressible. POSIX's rules require comparing full submatch sets between threads and are notoriously hard to implement correctly.

---

## What I'd change

- *Capture groups.* The `Save` instruction is intended for, but not implemented, using capture groups.

- *Better error messages* - Currently, errors report a token index, rather than a character offset in the input pattern.

- *Add a lazy DFA Cache* - In comparison with the actual Rust regex crate, mine loses by a substantial amount, one of the reasons is because I dont have a DFA cache so repeated values keep getting computed.

---


## References

- Russ Cox, [*Regular Expression Matching Can Be Simple And Fast*](https://swtch.com/~rsc/regexp/regexp1.html) (2007) — Thompson's construction, the NFA simulation, the DFA cache
- Russ Cox, [*Regular Expression Matching: the Virtual Machine Approach*](https://swtch.com/~rsc/regexp/regexp2.html) (2009) — the bytecode framing, submatch tracking, thread priority
- Russ Cox, [*Regular Expression Matching in the Wild*](https://swtch.com/~rsc/regexp/regexp3.html) (2010) — a tour of RE2
- Ken Thompson, *Regular Expression Search Algorithm*, CACM 11(6), 1968 — the original
- Bob Nystrom, [*Crafting Interpreters*](https://craftinginterpreters.com/) — recursive descent parsing (chapters 4–6)
---

## Status

I built this as a learning project to construct automata-based matching from scratch. This works and is tested, but it's not meant to be a production-ready substitute for the regex crate.
