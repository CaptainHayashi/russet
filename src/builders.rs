//! Builder functions for common tokeniser configurations.

use std::collections::hashmap::HashMap;

use tokeniser::{
    Error,
    Tokeniser,
    IgnoreEscapes,
    ParseEscapes,
    QuoteMode
};


/// A type for tokenisers returned by Russet builders.
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
    use super::{ unpack, whitespace_split_tokeniser };
    use tokeniser::{
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
