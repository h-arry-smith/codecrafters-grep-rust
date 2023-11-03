use std::env;
use std::io;
use std::ops::Not;
use std::process;

struct Matcher {
    fragments: Vec<Match>,
    match_from_start: bool,
}

impl Matcher {
    fn from_pattern(mut pattern: &str) -> Self {
        let match_from_start = pattern.starts_with('^');

        if match_from_start {
            pattern = &pattern[1..];
        }

        let fragments = Self::parse_pattern(pattern);

        Self {
            fragments,
            match_from_start,
        }
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
            Some('\\') => fragments.push(Match::Literal('\\'.to_string())),
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
        let mut char_index = 0;
        let mut fragments = self.fragments.iter();
        let mut current_fragment = fragments.next();

        loop {
            // We are out of fragments, so the pattern has matched
            if current_fragment.is_none() {
                return true;
            }

            // We are out of string, but still have fragments, so we didn't match
            if char_index >= input_line.len() {
                return false;
            }

            let fragment = current_fragment.unwrap();

            match fragment.r#match(input_line, &char_index) {
                MatchResult::Match(match_length) => {
                    // The fragment matched, so we can get the next fragment
                    current_fragment = fragments.next();
                    char_index += match_length;
                }
                MatchResult::NoMatch => {
                    // If match_from_start is true, then we fail here
                    if self.match_from_start {
                        return false;
                    } else {
                        // The fragment didn't match, so we advance the char_index and try again
                        char_index += 1;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Match {
    Literal(String),
    Class(Class),
    PositiveGroup(Vec<Match>),
    NegativeGroup(Vec<Match>),
}

enum MatchResult {
    Match(usize),
    NoMatch,
}

impl Not for MatchResult {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            MatchResult::Match(_) => MatchResult::NoMatch,
            MatchResult::NoMatch => MatchResult::Match(0),
        }
    }
}

impl From<bool> for MatchResult {
    fn from(b: bool) -> Self {
        if b {
            MatchResult::Match(1)
        } else {
            MatchResult::NoMatch
        }
    }
}

impl From<MatchResult> for bool {
    fn from(m: MatchResult) -> Self {
        match m {
            MatchResult::Match(_) => true,
            MatchResult::NoMatch => false,
        }
    }
}

impl Match {
    fn r#match(&self, input_line: &str, char_index: &usize) -> MatchResult {
        match self {
            Match::Literal(literal) => {
                let literal_length = literal.len();

                if input_line.len() < literal_length {
                    return MatchResult::NoMatch;
                }

                let input_line_fragment = &input_line[*char_index..*char_index + literal_length];

                if input_line_fragment != *literal {
                    return MatchResult::NoMatch;
                }

                MatchResult::Match(literal_length)
            }
            Match::Class(class) => match class {
                // TODO: Very similar code, should be able to generalize with a high order function
                Class::Digit => input_line[*char_index..]
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_digit()
                    .into(),
                Class::Word => input_line[*char_index..]
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_alphanumeric()
                    .into(),
            },
            Match::PositiveGroup(group_fragments) => group_fragments
                .iter()
                .any(|fragment| fragment.r#match(input_line, char_index).into())
                .into(),
            Match::NegativeGroup(group_fragments) => group_fragments
                .iter()
                .all(|fragment| (!fragment.r#match(input_line, char_index)).into())
                .into(),
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
