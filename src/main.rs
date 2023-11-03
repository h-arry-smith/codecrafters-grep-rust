use std::env;
use std::io;
use std::process;

struct Matcher {
    fragments: Vec<Match>,
}

impl Matcher {
    fn from_pattern(pattern: &str) -> Self {
        let fragments = Self::parse_pattern(pattern);
        Self { fragments }
    }

    fn parse_pattern(pattern: &str) -> Vec<Match> {
        let mut fragments = Vec::new();
        let mut chars = pattern.chars();

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    Self::parse_character_class(&mut chars, &mut fragments);
                }
                '[' => {
                    Self::parse_positive_character_group(&mut chars, &mut fragments);
                }
                c => fragments.push(Match::Literal(c.to_string())),
            }
        }

        fragments
    }

    fn parse_character_class(chars: &mut std::str::Chars, fragments: &mut Vec<Match>) {
        match chars.next() {
            Some('d') => fragments.push(Match::Class(Class::Digit)),
            Some('w') => fragments.push(Match::Class(Class::Word)),
            Some(c) => todo!("Handle character class: {}", c),
            None => panic!("Expected character after '\\'"),
        }
    }

    fn parse_positive_character_group(chars: &mut std::str::Chars, fragments: &mut Vec<Match>) {
        let mut group = Vec::new();
        let mut group_negative = false;

        for c in chars.by_ref() {
            match c {
                // TODO: This can only be the first character in the group, should be an error otherwise
                '^' => {
                    group_negative = true;
                }
                ']' => {
                    if group_negative {
                        fragments.push(Match::NegativeGroup(group));
                    } else {
                        fragments.push(Match::PositiveGroup(group));
                    }
                    return;
                }
                // TODO: not gauranteed to be a literal, should use the parse function, but is regex recursive?
                c => group.push(Match::Literal(c.to_string())),
            }
        }
    }

    fn r#match(&self, input_line: &str) -> bool {
        self.fragments
            .iter()
            .all(|fragment| fragment.r#match(input_line))
    }
}

#[derive(Debug)]
enum Match {
    Literal(String),
    Class(Class),
    PositiveGroup(Vec<Match>),
    NegativeGroup(Vec<Match>),
}

impl Match {
    fn r#match(&self, input_line: &str) -> bool {
        match self {
            Match::Literal(literal) => input_line.contains(literal),
            Match::Class(class) => match class {
                Class::Digit => input_line.chars().any(|c| c.is_ascii_digit()),
                Class::Word => input_line.chars().any(|c| c.is_ascii_alphanumeric()),
            },
            Match::PositiveGroup(group_fragments) => group_fragments
                .iter()
                .any(|fragment| fragment.r#match(input_line)),
            Match::NegativeGroup(group_fragments) => group_fragments
                .iter()
                .all(|fragment| !fragment.r#match(input_line)),
        }
    }
}

#[derive(Debug)]
enum Class {
    Digit,
    Word,
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let matcher = Matcher::from_pattern(pattern);
    matcher.r#match(input_line)
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
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
