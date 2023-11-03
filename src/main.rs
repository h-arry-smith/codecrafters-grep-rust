use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut chars = pattern.chars();
    if pattern.len() == 1 {
        input_line.contains(pattern)
    } else if chars.next().unwrap() == '\\' {
        if chars.next().unwrap() == 'd' {
            input_line.chars().any(|c| c.is_ascii_digit())
        } else {
            todo!("\\{} pattern not implemented yet", chars.nth(1).unwrap());
        }
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
