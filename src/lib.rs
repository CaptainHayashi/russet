//! # Russet
//!
//! __Russet__ is a simple string tokenising library for Rust.  It supports
//! POSIX shell-style separation of a line of _words_ into a vector of
//! strings.
#![experimental]

#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;

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

    /// The current closing quote character and quote mode, if any.
    quote: Option<( char, QuoteMode )>,

    /// Whether the tokeniser is currently processing an escape character.
    escaping: bool,

    /// Maps from quote openers to quote closers.
    quote_pairs: Q,

    /// Map from escape characters to their replacements.
    escape_pairs: E,

    /// The character preceding escape characters.
    escape_leader: Option<char>
}


/// A quote mode.
#[deriving(Clone)]
pub enum QuoteMode {
    /// All characters except the closing character have their literal value.
    /// This is equivalent to single-quoting in POSIX shell.
    IgnoreEscapes,

    /// All characters except the closing character and escape sequences
    /// have their literal value.  This is roughly equivalent to
    /// double-quoting in POSIX shell.
    ParseEscapes
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


impl<Q: Map<char, ( char, QuoteMode )>+Collection+Clone,
     E: Map<char, char>+Clone> Tokeniser<Q, E> {
    /// Creates a new, blank Tokeniser.
    ///
    /// # Arguments
    ///
    /// * `quote_pairs`   - A map, mapping characters that serve as opening
    ///                     quotes to their closing quotes and quote modes.
    /// * `escape_pairs`  - A map, mapping escape characters to the characters
    ///                     they represent.  An empty map is treated as a
    ///                     _shell-style escape_: the
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
    /// use russet::{ Tokeniser, ParseEscapes, QuoteMode };
    ///
    /// let quote_pairs: HashMap<char, ( char, QuoteMode )> =
    ///     vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
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
    /// use russet::whitespace_split_tokeniser;
    ///
    /// let tok = whitespace_split_tokeniser();
    /// let tok2 = tok.add_char('a').add_char('b').add_char('c');
    /// assert_eq!(tok2.into_strings(), Ok(vec![ "abc".into_string() ]));
    /// ```
    pub fn add_char(self, chr: char) -> Tokeniser<Q, E> {
        let mut new = self.clone();

        match (chr, self) {
            // ESCAPE SEQUENCES
            //   Shell-style escaping (no escapes defined)
            //   -> Echo character
            ( c, Tokeniser { escaping: true, escape_pairs: ref es, .. } )
                if es.is_empty() => new.emit(c),
            // Known escaped character, otherwise
            // -> Escape character
            ( e, Tokeniser { escaping: true, escape_pairs: ref es, .. } )
                if es.contains_key(&e) => {
                let x = es.find(&e).unwrap();

                new.emit(x.clone());
            },

            // ESCAPE LEADER
            //   Escape leader, not in quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser {
                escaping: false,
                quote: None,
                escape_leader: Some(e),
                ..
            } ) if e == c => new.start_escaping(),
            //   Escape leader, in escape-permitting quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser {
                escaping: false,
                quote: Some(( _, ParseEscapes )),
                escape_leader: Some(e),
                ..
            } ) if e == c => new.start_escaping(),

            // QUOTE OPENING
            //   Quote opening character, not currently in quoted word
            //   -> Start quoting
            ( q, Tokeniser {
                escaping: false,
                quote: None,
                quote_pairs: ref qs,
                ..
            } ) if qs.contains_key(&q) => {
                new.quote = Some(qs.find(&q).unwrap().clone());
                new.in_word = true;
            },

            // QUOTE CLOSING
            //   Quote closing character, in quoted word, quotes ok
            //   -> Stop quoting
            ( c, Tokeniser { escaping: false, quote: Some(( cc, _ )), .. } )
                if c == cc => {
                new.quote = None;
                new.in_word = true;
            },

            // UNESCAPED WHITESPACE
            //   Unescaped whitespace, while not in a word
            //   -> Ignore
            ( a, Tokeniser { escaping: false, in_word: false, .. } )
                if is_whitespace(a) => (),
            //   Unescaped whitespace, while in a non-quoted word
            //   -> End word
            ( a, Tokeniser { escaping: false, in_word: true, quote: None, .. } )
                if is_whitespace(a) => {
                new.in_word = false;
                new.vec.push(String::new());
            },

            // DEFAULT
            //   Anything else
            //   -> Echo
            ( a, _ ) => new.emit(a)
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
            self.drop_empty_current_string();
            Ok(self.vec)
        }
    }

    /// Adds a character into a Tokeniser's current string.
    /// This automatically sets the Tokeniser's state to be in a word,
    /// and clears any escape sequence flag.
    fn emit(&mut self, c: char) {
        self.in_word = true;
        self.escaping = false;
        self.vec.mut_last().mutate(|s| { s.push_char(c); s });
    }

    /// Switches on escape mode.
    /// This automatically sets the Tokeniser to be in a word, if it isn't
    /// already.
    fn start_escaping(&mut self) {
        self.escaping = true;
        self.in_word = true;
    }

    /// Drops the current working string, if it is empty.
    fn drop_empty_current_string(&mut self) {
        if self.vec.last().map(|s| s.is_empty()).unwrap_or(false) {
            self.vec.pop();
        }
    }
}


/// A type for tokenisers returned by Russet.
pub type StockTokeniser = Tokeniser<HashMap<char, ( char, QuoteMode )>,
                                    HashMap<char, char>>;


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
///                                         "\\slashes".into_string())));
/// ```
#[experimental]
pub fn whitespace_split_tokeniser() -> StockTokeniser {
    let quote_pairs: HashMap<char, ( char, QuoteMode )> = HashMap::new();
    let escape_pairs: HashMap<char, char> = HashMap::new();
    Tokeniser::new(quote_pairs, escape_pairs, None)
}


/// Creates a Tokeniser that provides shell-style quoting.
///
/// This recognises pairs of " and ' as delineating words, and parses
/// anything following a \ as its literal value.  Anything in single quotes
/// is returned verbatim.
///
/// # Return value
///
/// A Tokeniser with shell-style quoting.
///
/// # Example
///
/// ```rust
/// use russet::shell_style_tokeniser;
///
/// let tok = shell_style_tokeniser();
/// let tok2 = tok.add_line("word1 word\\ 2 \"word\\ 3\" 'word\\ \"4\"'");
/// assert_eq!(tok2.into_strings(), Ok(vec!("word1".into_string(),
///                                         "word 2".into_string(),
///                                         "word 3".into_string(),
///                                         "word\\ \"4\"".into_string())));
/// ```
#[experimental]
pub fn shell_style_tokeniser() -> StockTokeniser {
    let quote_pairs: HashMap<char, ( char, QuoteMode )> =
        vec![ ( '\"', ( '\"', ParseEscapes ) ),
              ( '\'', ( '\'', IgnoreEscapes ) ) ].move_iter().collect();
    let escape_pairs: HashMap<char, char> = HashMap::new();
    Tokeniser::new(quote_pairs, escape_pairs, Some('\\'))
}


/// Unpacks a line into its constituent words.
#[experimental]
pub fn unpack(line: &str) -> Result<Vec<String>, Error> {
    let quote_pairs: HashMap<char, ( char, QuoteMode )> =
        vec![ ( '\"', ( '\"', ParseEscapes ) ),
              ( '\'', ( '\'', ParseEscapes ) ) ].move_iter().collect();
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
        whitespace_split_tokeniser,
        Error,
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

    /// The whitespace_split_tokeniser should provide the same strings as
    /// the Words iterator for an arbitrary string.
    #[quickcheck]
    fn whitespace_split_tokeniser_words_equivalence(line : String) -> bool {
        let line_slice = line.as_slice();

        let lhs: Result<Vec<String>, Error> =
            whitespace_split_tokeniser().add_line(line_slice).into_strings();
        let rhs: Result<Vec<String>, Error> =
            Ok(line_slice.words().map(|x| x.into_string()).collect());

        lhs == rhs
    }
}
