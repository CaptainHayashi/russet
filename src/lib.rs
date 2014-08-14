//! # Russet
//!
//! __Russet__ is a simple string tokenising library for Rust.  It supports
//! POSIX shell-style separation of a line of _words_ into a vector of
//! strings.
#![experimental]

use std::char::is_whitespace;


/// A tokeniser object.
///
/// A Tokeniser can be fed characters from an iterator, string, or individually.
/// It is an _immutable_ object: actions on a Tokeniser consume the Tokeniser,
/// and produce a fresh copy of the Tokeniser.
///
/// At any stage, a Tokeniser can be consumed to produce the vector of words
/// it has read, using the `into_strings` method.  This method may fail if the
/// Tokeniser ended in a bad state (in the middle of a quoted string, or in
/// the middle of an escape sequence).
#[deriving(Clone)]
pub struct Tokeniser {
    vec:       Vec<String>,
    in_word:   bool,
    in_quoted: bool,
    in_escape: bool
}


/// A tokeniser error.
///
/// A Tokeniser's `into_strings` method can fail with one of the following
/// errors if called while the Tokeniser is in an unfinished state.
#[deriving(Eq, PartialEq, Show)]
pub enum Error {
    /// A quotation was opened, but not closed.
    UnmatchedQuote,

    /// An escape sequence was started, but not finished.
    UnfinishedEscape
}


impl Tokeniser {
    /// Creates a new, blank Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, with an empty state.  Attempting to take the
    /// string vector of the Tokeniser yields the empty vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russet::Tokeniser;
    ///
    /// let tok = Tokeniser::new();
    /// assert_eq!(tok.into_strings(), Ok(vec![]));
    /// ```
    pub fn new() -> Tokeniser {
        Tokeniser {
            vec:       vec![ String::new() ],
            in_word:   false,
            in_quoted: false,
            in_escape: false
        }
    }

    /// Feeds a single character `chr` to a Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `chr`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russet::Tokeniser;
    ///
    /// let tok = Tokeniser::new().add_char('a').add_char('b').add_char('c');
    /// assert_eq!(tok.into_strings(), Ok(vec![ "abc".into_string() ]));
    /// ```
    pub fn add_char(self, chr: char) -> Tokeniser {
        let mut new = self.clone();

        match (chr, self) {
            // Escape character while not escaped
            // -> Begin escape (and word if not in one already)
            ( '\\', Tokeniser { in_escape: false, .. } ) => {
                new.in_escape = true;
                new.in_word   = true;
            },
            // Unescaped quote character
            // -> Toggle quoting
            ( '"', Tokeniser { in_escape: false, .. } ) => {
                new.in_quoted = !new.in_quoted;
                new.in_word   = true;
            },
            // Unescaped whitespace, while not in a word
            // -> Ignore
            ( a, Tokeniser { in_escape: false, in_word: false, .. } )
                if is_whitespace(a) => (),
            // Unescaped whitespace, while in a non-quoted word
            // -> End word
            ( a, Tokeniser {
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
            ( 'n', Tokeniser { in_escape: true, .. } ) => {
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
    ///
    /// # Return value
    ///
    /// A Result, containing the tokenised string vector if the Tokeniser
    /// was in a valid ending state, and an Error otherwise.
    pub fn into_strings(mut self) -> Result<Vec<String>, Error> {
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
        Tokeniser::new(), |s, chr| s.add_char(chr)
    ).into_strings()
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
