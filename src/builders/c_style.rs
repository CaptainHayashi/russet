//! Builder for the C-style stock tokeniser.
#![experimental]

use builders::types::{
    StockEscapeMap,
    StockQuoteMap,
    StockTokeniser
};
use escape_scheme::c_escapes;
use tokeniser::{ Tokeniser, ParseEscapes };


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
    let quote_map: StockQuoteMap =
        vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
    let escape_map: StockEscapeMap =
        vec![ ( '\\', c_escapes() ) ].move_iter().collect();
    Tokeniser::new(quote_map, escape_map)
}


#[cfg(test)]
mod test {
    use super::c_style_tokeniser;
    use line::LineTokeniser;
    use tokeniser::{ UnmatchedQuote, UnfinishedEscape };

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
}
