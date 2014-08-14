//! # Russet
//!
//! __Russet__ is a simple string tokenising library for Rust.  It supports
//! POSIX shell-style separation of a line of _words_ into a vector of
//! strings.
#![experimental]

use std::char::is_whitespace;
use std::collections::hashmap::HashMap;


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
pub struct Tokeniser<Q, E> {
    /// The current vector of parsed words.
    vec: Vec<String>,

    /// Whether or not we are currently in a word.
    in_word: bool,

    /// The current closing quote character, if any.
    quote: Option<char>,

    /// Whether the tokeniser is currently processing an escape character.
    escaping: bool,

    /// Maps from quote openers to quote closers.
    quote_pairs: Q,

    /// Map from escape characters to their replacements.
    escape_pairs: E,

    /// The character preceding escape characters.
    escape_leader: Option<char>
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


impl<Q: Map<char, char>+Clone, E: Map<char, char>+Clone> Tokeniser<Q, E> {
    /// Creates a new, blank Tokeniser.
    ///
    /// # Arguments
    ///
    /// * `quote_pairs`   - A map, mapping characters that serve as opening
    ///                     quotes to their closing quotes.
    /// * `escape_pairs`  - A map, mapping escape characters to the characters
    ///                     they represent.
    /// * `escape_leader` - The character, if any, that starts an escape
    ///                     sequence.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, with an empty state.  Attempting to take the
    /// string vector of the Tokeniser yields the empty vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::hashmap::HashMap;
    /// use russet::Tokeniser;
    ///
    /// let quote_pairs: HashMap<char, char> =
    ///     vec![ ( '\"', '\"' ), ( '\'', '\'' ) ].move_iter().collect();
    /// let escape_pairs: HashMap<char, char> =
    ///     vec![ ( 'n', '\n' ) ].move_iter().collect();
    /// let tok = Tokeniser::new(quote_pairs, escape_pairs, Some('\\'));
    /// assert_eq!(tok.into_strings(), Ok(vec![]));
    /// ```
    pub fn new(quote_pairs: Q, escape_pairs: E, escape_leader: Option<char>)
               -> Tokeniser<Q, E> {
        Tokeniser {
            vec: vec![ String::new() ],
            in_word: false,
            quote: None,
            escaping: false,
            quote_pairs: quote_pairs,
            escape_pairs: escape_pairs,
            escape_leader: escape_leader
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
    /// use std::collections::hashmap::HashMap;
    ///
    /// let quote_pairs: HashMap<char, char> =
    ///     vec![ ( '\"', '\"' ), ( '\'', '\'' ) ].move_iter().collect();
    /// let escape_pairs: HashMap<char, char> =
    ///     vec![ ( 'n', '\n' ) ].move_iter().collect();
    /// let tok = Tokeniser::new(quote_pairs, escape_pairs, Some('\\'));
    /// let tok2 = tok.add_char('a').add_char('b').add_char('c');
    /// assert_eq!(tok2.into_strings(), Ok(vec![ "abc".into_string() ]));
    /// ```
    pub fn add_char(self, chr: char) -> Tokeniser<Q, E> {
        let mut new = self.clone();

        match (chr, self) {
            // Escape character while not escaped
            // -> Begin escape (and word if not in one already)
            ( '\\', Tokeniser { escaping: false, .. } ) => {
                new.escaping = true;
                new.in_word = true;
            },
            // Unescaped quote opening character, not currently in quoted word
            // -> Start quoting
            ( q, Tokeniser {
                escaping: false,
                quote: None,
                quote_pairs: ref qs,
                ..
            } ) if qs.contains_key(&q) => {
                new.quote = Some(qs.find(&q).unwrap().clone());
                new.in_word = true;
            },
            // Unescaped quote closing character, in quoted word, quotes ok
            // -> Stop quoting
            ( c, Tokeniser { escaping: false, quote: Some(cc), .. } )
                if c == cc => {
                new.quote = None;
                new.in_word = true;
            },
            // Unescaped whitespace, while not in a word
            // -> Ignore
            ( a, Tokeniser { escaping: false, in_word: false, .. } )
                if is_whitespace(a) => (),
            // Unescaped whitespace, while in a non-quoted word
            // -> End word
            ( a, Tokeniser { escaping: false, in_word: true, quote: None, .. } )
                if is_whitespace(a) => {
                new.in_word = false;
                new.vec.push(String::new());
            },
            // Known escaped character
            // -> Escape character
            ( e, Tokeniser { escaping: true, escape_pairs: ref es, .. } )
                if es.contains_key(&e) => {
                let x = es.find(&e).unwrap();

                new.in_word = true;
                new.escaping = false;
                new.vec.mut_last().mutate(|s| { s.push_char(x.clone()); s });
            },
            // Anything else
            // -> Echo
            ( a, _ ) => {
                new.in_word = true;
                new.escaping = false;
                new.vec.mut_last().mutate(|s| { s.push_char(a); s });
            }
        }

        new
    }

    /// Feeds a line into the Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `line`.
    pub fn add_line(self, line: &str) -> Tokeniser<Q, E> {
        line.trim().chars().fold(self, |s, chr| s.add_char(chr))
    }

    /// Destroys the tokeniser, extracting the string vector.
    ///
    /// # Return value
    ///
    /// A Result, containing the tokenised string vector if the Tokeniser
    /// was in a valid ending state, and an Error otherwise.
    pub fn into_strings(mut self) -> Result<Vec<String>, Error> {
        if self.in_word && self.quote.is_some() {
            Err(UnmatchedQuote)
        } else if self.escaping {
            Err(UnfinishedEscape)
        } else {
            if self.vec.last().map(|s| s.len() == 0).unwrap_or(false) {
                self.vec.pop();
            }

            Ok(self.vec)
        }
    }
}


/// A type for tokenisers returned by Russet.
pub type StockTokeniser = Tokeniser<HashMap<char, char>, HashMap<char, char>>;


/// Creates a Tokeniser that doesn't support quoting or escaping.
///
/// This is effectively equivalent to the Words iterator on string slices.
///
/// # Return value
///
/// A Tokeniser with no quoting or escaping.
///
/// # Example
///
/// ```rust
/// use russet::whitespace_split_tokeniser;
///
/// let tok = whitespace_split_tokeniser();
/// let tok2 = tok.add_line("this \"ignores quotes\"  and\n  \\slashes");
/// assert_eq!(tok2.into_strings(), Ok(vec!("this".into_string(),
///                                         "\"ignores".into_string(),
///                                         "quotes\"".into_string(),
///                                         "and".into_string(),
///                                         "slashes".into_string())));
/// ```
#[experimental]
pub fn whitespace_split_tokeniser() -> StockTokeniser {
    let quote_pairs: HashMap<char, char> = HashMap::new();
    let escape_pairs: HashMap<char, char> = HashMap::new();
    Tokeniser::new(quote_pairs, escape_pairs, None)
}


/// Unpacks a line into its constituent words.
#[experimental]
pub fn unpack(line: &str) -> Result<Vec<String>, Error> {
    let quote_pairs: HashMap<char, char> =
        vec![ ( '\"', '\"' ), ( '\'', '\'' ) ].move_iter().collect();
    let escape_pairs: HashMap<char, char> =
        vec![ ( 'n', '\n' ) ].move_iter().collect();

    line.trim().chars().fold(
        Tokeniser::new(quote_pairs, escape_pairs, Some('\\')),
        |s, chr| s.add_char(chr)
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
