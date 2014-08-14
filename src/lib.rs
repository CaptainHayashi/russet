//! # Russet
//!
//! __Russet__ is a simple string tokenising library for Rust.  It supports
//! POSIX shell-style separation of a line of _words_ into a vector of
//! strings.
#![experimental]

use std::char::is_whitespace;


#[deriving(Clone)]
struct TokeniserState {
    vec:       Vec<String>,
    in_word:   bool,
    in_quoted: bool,
    in_escape: bool
}


/// A tokeniser error.
#[deriving(Eq, PartialEq, Show)]
pub enum Error {
    /// A quotation was opened, but not closed.
    UnmatchedQuote,

    /// An escape sequence was started, but not finished.
    UnfinishedEscape
}


impl TokeniserState {
    /// Creates a new, blank TokeniserState.
    fn new() -> TokeniserState {
        TokeniserState {
            vec:       vec![ String::new() ],
            in_word:   false,
            in_quoted: false,
            in_escape: false
        }
    }

    // Performs one step of the tokeniser.
    fn step(self, chr: char) -> TokeniserState {
        let mut new = self.clone();

        match (chr, self) {
            // Escape character while not escaped
            // -> Begin escape (and word if not in one already)
            ( '\\', TokeniserState { in_escape: false, .. } ) => {
                new.in_escape = true;
                new.in_word   = true;
            },
            // Unescaped quote character
            // -> Toggle quoting
            ( '"', TokeniserState { in_escape: false, .. } ) => {
                new.in_quoted = !new.in_quoted;
                new.in_word   = true;
            },
            // Unescaped whitespace, while not in a word
            // -> Ignore
            ( a, TokeniserState { in_escape: false, in_word: false, .. } )
                if is_whitespace(a) => (),
            // Unescaped whitespace, while in a non-quoted word
            // -> End word
            ( a, TokeniserState {
                in_escape: false,
                in_word:   true,
                in_quoted: false,
                ..
            } ) if is_whitespace(a) => {
                new.in_word = false;
                new.vec.push(String::new());
            },
            // Escaped n
            // -> Newline
            ( 'n', TokeniserState { in_escape: true, .. } ) => {
                new.in_word   = true;
                new.in_escape = false;
                new.vec.mut_last().mutate(|s| { s.push_char('\n'); s });
            },
            // Anything else
            // -> Echo
            ( a, _ ) => {
                new.in_word   = true;
                new.in_escape = false;
                new.vec.mut_last().mutate(|s| { s.push_char(a); s });
            }
        }

        new
    }

    /// Destroys the tokeniser, extracting the string vector.
    fn unwrap(mut self) -> Result<Vec<String>, Error> {
        if self.in_word && self.in_quoted {
            Err(UnmatchedQuote)
        } else if self.in_escape {
            Err(UnfinishedEscape)
        } else {
            if self.vec.last().map(|s| s.len() == 0).unwrap_or(false) {
                self.vec.pop();
            }

            Ok(self.vec)
        }
    }
}


/// Unpacks a line into its constituent words.
#[experimental]
pub fn unpack(line: &str) -> Result<Vec<String>, Error> {
    line.trim().chars().fold(
        TokeniserState::new(), |s, chr| s.step(chr)
    ).unwrap()
}


#[cfg(test)]
mod test {
    use super::{
        unpack,
        UnmatchedQuote,
        UnfinishedEscape
    };

    #[test]
    fn unpack_unmatched_quote() {
        assert_eq!(unpack("\"abcde"), Err(UnmatchedQuote));
    }

    #[test]
    fn unpack_unfinished_escape() {
        assert_eq!(unpack("zxcvbn m\\"), Err(UnfinishedEscape));
    }

    #[test]
    fn unpack_empty_string() {
        assert_eq!(unpack(""), Ok(vec![]));
    }

    #[test]
    fn unpack_leading_whitespace() {
        let rhs = vec![ "abc".into_string(), "def".into_string() ];
        assert_eq!(unpack("     abc def"), Ok(rhs));
    }

    #[test]
    fn unpack_trailing_whitespace() {
        let rhs = vec![ "ghi".into_string(), "jkl".into_string() ];
        assert_eq!(unpack("ghi jkl     \n"), Ok(rhs));
    }

    #[test]
    fn unpack_enqueue_command() {
        let lhs =
            "enqueue file \"C:\\\\Users\\\\Test\\\\Artist - Title.mp3\" 1";
        let rhs = vec![ "enqueue".into_string(),
                        "file".into_string(),
                        "C:\\Users\\Test\\Artist - Title.mp3".into_string(),
                        "1".into_string() ];
        assert_eq!(unpack(lhs), Ok(rhs));
    }

    #[test]
    fn unpack_escaped_newline() {
        assert_eq!(unpack("abc\\nde"),     Ok(vec![ "abc\nde".into_string() ]));
        assert_eq!(unpack("\"abc\\nde\""), Ok(vec![ "abc\nde".into_string() ]));
    }
}
