use std::vec;

use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

impl<'a> MatcherToken<'a> {
    fn check_string(&self, text: &'a str) -> (bool, &'a str, &'a str) {
        if text.is_empty() {
            return (false, "", "");
        }

        match self {
            MatcherToken::RawText(string) => {
                let string_len = string.len();

                // FIXME
                eprintln!("raw text inside: {string}");
                eprintln!("raw text given: {text}");
                let case1 = string_len < text.len();
                eprintln!("less than: {case1}");

                if string_len > text.len() {
                    return (false, "", "");
                }

                let text_test = &text[..string_len];

                // FIXME
                let case2 = !(*string).eq(text_test);
                eprintln!("not equal: {case2}");

                if !(*string).eq(text_test) {
                    (false, "", "")
                } else {
                    (true, &text[string_len..], string)
                }
            }
            // FIXME: need to parse with string length (not entire string)
            MatcherToken::OneOfText(vec) => {
                // need some initial value
                let mut text_match = "";
                let check = vec.iter().any(|string| {
                    text_match = *string;
                    text_match == &text[..text_match.len()]
                });
                if check {
                    (check, &text[text_match.len()..], text_match)
                } else {
                    (check, "", "")
                }
            }
            MatcherToken::WildCard => {
                if text.is_empty() {
                    (false, "", "")
                } else {
                    (true, &text[1..], &text[..1])
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

const INDEX_ERROR: usize = usize::MAX;

/// FIXME: do not assume unicode chars...
impl<'a> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut tokens: Vec<MatcherToken> = Vec::new();
        let mut current_text = text;

        // UPDATE: needed to keep track if the matching breaked at any point...
        let mut match_break = false;

        loop {
            // UPDATE: need to break if everything is matched successfully (current_text is empty)
            if current_text.is_empty() {
                break;
            }

            let (match_token_option, remaining_text) = find_match_token(current_text);
            match match_token_option {
                Some(match_token) => {
                    tokens.push(match_token);
                    current_text = remaining_text;
                }
                None => {
                    match_break = true;
                    break
                },
            }
        }

        if match_break {
            None
        } else {
            Some(Matcher {
                text: text,
                tokens: tokens,
                most_tokens_matched: 0,
            })
        }
    }

    /// FIXME: self should be immutable? -- need its own lifetime...
    /// FIXME: needed lifetime bound 'a: 'c...
    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string<'b, 'c>(&'b mut self, string: &'c str) -> Vec<(&'b MatcherToken<'a>, &'c str)>
    where
        'a: 'c,
    {
        // UPDATE: need to reset most_tokens_matched to 0 again (may be called multiple times on different strings...)
        self.most_tokens_matched = 0;
        let mut matcher_token_vec = vec![];
        let mut current_text = string;
        for match_token in &self.tokens {
            let (check, remaining_text, match_string) = match_token.check_string(&current_text);
            if check {
                self.most_tokens_matched += 1;
                current_text = remaining_text;

                matcher_token_vec.push((match_token, match_string));
            } else {
                break;
            }
        }
        
        // FIXME
        eprintln!("final matcher token vec: {matcher_token_vec:?}");
        matcher_token_vec
    }
}

fn find_match_token<'a>(text: &'a str) -> (Option<MatcherToken>, &str) {
    let mut text_iter = text.chars();

    match text_iter.next() {
        Some(first_char) => match first_char {
            '(' => {
                let mut one_of_text_str_vec: Vec<&str> = Vec::new();
                // need to start from beginning or relative?
                let right_paren_index = text[..].find(')').unwrap_or(INDEX_ERROR);
                if right_paren_index == INDEX_ERROR {
                    (None, "")
                } else {
                    // ignore first '('
                    let mut left_start_index = 1;
                    let mut intermediate_str;
                    loop {
                        intermediate_str = &text[left_start_index..right_paren_index];
                        let separator_index = intermediate_str.find('|').unwrap_or(INDEX_ERROR);

                        // FIXME:
                        eprintln!("intermediate string: {intermediate_str}");
                        eprintln!("separator index: {separator_index}");

                        if separator_index == INDEX_ERROR {
                            let one_of_text_single_str = &text[left_start_index..right_paren_index];
                            one_of_text_str_vec.push(one_of_text_single_str);
                            break;
                        } else {
                            // update with separator index offset (from relative to absolute)
                            let one_of_text_single_str = &text[left_start_index..left_start_index + separator_index];
                            one_of_text_str_vec.push(one_of_text_single_str);
                            // update with separator index offset (from relative to absolute)
                            left_start_index = left_start_index + separator_index + 1;
                        }
                    }
                    (
                        Some(MatcherToken::OneOfText(one_of_text_str_vec)),
                        &text[right_paren_index + 1..],
                    )
                }
            }
            '.' => (Some(MatcherToken::WildCard), &text[1..]),
            _ => {
                // FIXME!: need to find min of both locations...
                let left_paren_index = text.find('(').unwrap_or(INDEX_ERROR);
                if left_paren_index != INDEX_ERROR {
                    (
                        Some(MatcherToken::RawText(&text[..left_paren_index])),
                        &text[left_paren_index..],
                    )
                } else {
                    let wildcard_index = text.find('.').unwrap_or(INDEX_ERROR);
                    if wildcard_index != INDEX_ERROR {
                        (
                            Some(MatcherToken::RawText(&text[..left_paren_index])),
                            &text[wildcard_index..],
                        )
                    } else {
                        (Some(MatcherToken::RawText(&text[..])), "")
                    }
                }
            }
        },
        None => (None, ""),
    }
}

/// FIXME: copied from README of ex06
/// UPDATE: not needed after all (WordIterator)
/// This struct keeps track of where we're up to in the string.
// struct WordIterator<'s> {
//     position: usize,
//     string: &'s str,
// }

// impl<'lifetime> WordIterator<'lifetime> {
//     /// Creates a new WordIterator based on a string.
//     fn new(string: &'lifetime str) -> WordIterator<'lifetime> {
//         WordIterator {
//             position: 0,
//             string,
//         }
//     }

//     /// Gives the next word. `None` if there aren't any words left.
//     fn next_word(&mut self) -> Option<&str> {
//         let start_of_word = &self.string[self.position..];
//         let index_of_next_space = start_of_word.find(' ').unwrap_or(start_of_word.len());
//         if start_of_word.len() != 0 {
//             self.position += index_of_next_space + 1;
//             Some(&start_of_word[..index_of_next_space])
//         } else {
//             None
//         }
//     }
// }

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};

    //////////////////////////////////////////////////////////////////////////////////////////////
    /// Begin personal tests
    //////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn check_string_test() {
        let matcher_token_raw_text = MatcherToken::RawText("test");

        assert_eq!(
            matcher_token_raw_text.check_string("test123"),
            (true, "123", "test")
        );
        assert_eq!(matcher_token_raw_text.check_string("tes"), (false, "", ""));

        let matcher_token_one_of_text = MatcherToken::OneOfText(vec!["one", "two"]);

        assert_eq!(
            matcher_token_one_of_text.check_string("onea"),
            (true, "a", "one")
        );
        assert_eq!(
            matcher_token_one_of_text.check_string("twob"),
            (true, "b", "two")
        );
        assert_eq!(
            matcher_token_one_of_text.check_string("tes"),
            (false, "", "")
        );

        let matcher_token_wildcard = MatcherToken::WildCard;

        assert_eq!(matcher_token_wildcard.check_string(""), (false, "", ""));
        assert_eq!(
            matcher_token_wildcard.check_string("abc"),
            (true, "bc", "a")
        );
    }

    #[test]
    fn create_matcher_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(
            matcher,
            Matcher {
                text: &match_string,
                tokens: vec![
                    MatcherToken::RawText("abc"),
                    MatcherToken::OneOfText(vec!["d", "e", "f"]),
                    MatcherToken::WildCard
                ],
                most_tokens_matched: 0
            }
        );
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    /// End personal tests
    //////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcde".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "e") // or '💪'
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
