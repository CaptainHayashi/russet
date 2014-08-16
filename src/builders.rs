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


/// Creates a Tokeniser that provides C-style quoting.
///
/// This recognises pairs of " as delineating words, and parses
/// `\n`, `\r`, `\"`, `\'`, and `\t` as their C equivalents.
///
/// # Return value
///
/// A Tokeniser with C-style quoting.
///
/// # Example
///
/// ```rust
/// use russet::c_style_tokeniser;
///
/// let tok = c_style_tokeniser();
/// let tok2 = tok.add_line("word1\nword\\n2 \"word\n3\" \"word\\n4\"");
/// assert_eq!(tok2.into_strings(), Ok(vec!("word1".into_string(),
///                                         "word\n2".into_string(),
///                                         "word\n3".into_string(),
///                                         "word\n4".into_string())));
/// ```
#[experimental]
pub fn c_style_tokeniser() -> StockTokeniser {
    let quote_pairs: HashMap<char, ( char, QuoteMode )> =
        vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
    let escape_pairs: HashMap<char, char> =
        vec![ ( 'n',  '\n' ),
              ( 'r',  '\r' ),
              ( '\"', '\"' ),
              ( '\'', '\'' ),
              ( 't',  '\t' ) ].move_iter().collect();
    Tokeniser::new(quote_pairs, escape_pairs, Some('\\'))
}


pub trait LineTokeniser {
    fn line(self, ln: &str) -> Result<Vec<String>, Error>;
}

impl LineTokeniser for fn() -> StockTokeniser {
    fn line(self, ln: &str) -> Result<Vec<String>, Error> {
        self().add_line(ln).into_strings()
    }
}


#[cfg(test)]
mod test {
    use super::{
        LineTokeniser,
        c_style_tokeniser,
        whitespace_split_tokeniser
    };
    use tokeniser::{
        Error,
        UnmatchedQuote,
        UnfinishedEscape
    };

    #[test]
    fn c_style_unmatched_quote() {
        assert_eq!(c_style_tokeniser.line("\"abcde"), Err(UnmatchedQuote));
    }

    #[test]
    fn c_style_unfinished_escape() {
        assert_eq!(c_style_tokeniser.line("zxcvbn m\\"), Err(UnfinishedEscape));
    }

    #[test]
    fn c_style_empty_string() {
        assert_eq!(c_style_tokeniser.line(""), Ok(vec![]));
    }

    #[test]
    fn c_style_leading_whitespace() {
        let rhs = vec![ "abc".into_string(), "def".into_string() ];
        assert_eq!(c_style_tokeniser.line("     abc def"), Ok(rhs));
    }

    #[test]
    fn c_style_trailing_whitespace() {
        let rhs = vec![ "ghi".into_string(), "jkl".into_string() ];
        assert_eq!(c_style_tokeniser.line("ghi jkl     \n"), Ok(rhs));
    }

    #[test]
    fn c_style_enqueue_command() {
        let lhs =
            "enqueue file \"C:\\\\Users\\\\Test\\\\Artist - Title.mp3\" 1";
        let rhs = vec![ "enqueue".into_string(),
                        "file".into_string(),
                        "C:\\Users\\Test\\Artist - Title.mp3".into_string(),
                        "1".into_string() ];
        assert_eq!(c_style_tokeniser.line(lhs), Ok(rhs));
    }

    #[test]
    fn c_style_escaped_newline() {
        assert_eq!(c_style_tokeniser.line("abc\\nde"),
                   Ok(vec![ "abc\nde".into_string() ]));
        assert_eq!(c_style_tokeniser.line("\"abc\\nde\""),
                   Ok(vec![ "abc\nde".into_string() ]));
    }

    /// The whitespace_split_tokeniser should provide the same strings as
    /// the Words iterator for an arbitrary string.
    #[quickcheck]
    fn whitespace_split_tokeniser_words_equivalence(line : String) -> bool {
        let line_slice = line.as_slice();

        let lhs: Result<Vec<String>, Error> =
            whitespace_split_tokeniser.line(line_slice);
        let rhs: Result<Vec<String>, Error> =
            Ok(line_slice.words().map(|x| x.into_string()).collect());

        lhs == rhs
    }
}
