use std::env;
use std::io;
use std::ops::Not;
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
                // TODO: Maybe these have to be first/last and we should check that?
                '^' => {
                    let next_char = chars.next().unwrap().to_string();
                    fragments.push(Match::StartOfLine(Box::new(
                        Self::parse_pattern(&next_char).pop().unwrap(),
                    )));
                }
                '$' => {
                    let previous_fragment = fragments.pop().unwrap();
                    fragments.push(Match::EndOfLine(Box::new(previous_fragment)));
                }
                '+' => {
                    let previous_fragment = fragments.pop().unwrap();
                    fragments.push(Match::OneOfMore(Box::new(previous_fragment)));
                }
                '?' => {
                    let previous_fragment = fragments.pop().unwrap();
                    fragments.push(Match::ZeroOrOne(Box::new(previous_fragment)));
                }
                '.' => {
                    fragments.push(Match::AnyChar);
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
                    char_index += 1;
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
    StartOfLine(Box<Match>),
    EndOfLine(Box<Match>),
    OneOfMore(Box<Match>),
    ZeroOrOne(Box<Match>),
    AnyChar,
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
            Match::StartOfLine(fragment) => {
                let result = fragment.r#match(input_line, char_index);
                match result {
                    MatchResult::Match(_) => {
                        if *char_index == 0 {
                            result
                        } else {
                            MatchResult::NoMatch
                        }
                    }
                    MatchResult::NoMatch => result,
                }
            }
            Match::EndOfLine(fragment) => {
                let result = fragment.r#match(input_line, char_index);
                match result {
                    MatchResult::Match(match_length) => {
                        if *char_index + match_length == input_line.len() {
                            result
                        } else {
                            MatchResult::NoMatch
                        }
                    }
                    MatchResult::NoMatch => result,
                }
            }
            Match::OneOfMore(fragment) => {
                let mut match_length = 0;

                loop {
                    let new_index = *char_index + match_length;
                    let result = fragment.r#match(input_line, &new_index);

                    match result {
                        MatchResult::Match(fragment_match_length) => {
                            match_length += fragment_match_length;
                        }
                        MatchResult::NoMatch => {
                            if match_length == 0 {
                                return MatchResult::NoMatch;
                            } else {
                                return MatchResult::Match(match_length);
                            }
                        }
                    }
                }
            }
            Match::ZeroOrOne(fragment) => {
                let result = fragment.r#match(input_line, char_index);

                match result {
                    MatchResult::Match(match_length) => MatchResult::Match(match_length),
                    MatchResult::NoMatch => MatchResult::Match(0),
                }
            }
            Match::AnyChar => MatchResult::Match(1),
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
