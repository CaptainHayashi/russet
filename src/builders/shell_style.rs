//! Builder for the shell-style stock tokeniser.
#![experimental]

use builders::types::{
    StockEscapeMap,
    StockQuoteMap,
    StockTokeniser
};
use escape_scheme::LiteralEscape;
use tokeniser::{ Tokeniser, IgnoreEscapes, ParseEscapes };


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
    let quote_map: StockQuoteMap =
        vec![ ( '\"', ( '\"', ParseEscapes ) ),
              ( '\'', ( '\'', IgnoreEscapes ) ) ].move_iter().collect();
    let escape_map: StockEscapeMap =
        vec![ ( '\\', LiteralEscape ) ].move_iter().collect();
    Tokeniser::new(quote_map, escape_map)
}


#[cfg(test)]
mod test {
    use super::shell_style_tokeniser;
    use line::LineTokeniser;
    use tokeniser::{ UnmatchedQuote, UnfinishedEscape };

    #[test]
    fn shell_style_unmatched_single_quote() {
        assert_eq!(shell_style_tokeniser.line("\'abcde"),
                   Err(UnmatchedQuote));
    }

    #[test]
    fn shell_style_unmatched_double_quote() {
        assert_eq!(shell_style_tokeniser.line("\"abcde"),
                   Err(UnmatchedQuote));
    }

    #[test]
    fn shell_style_unfinished_escape() {
        assert_eq!(shell_style_tokeniser.line("zxcvbn m\\"),
                   Err(UnfinishedEscape));
    }

    #[test]
    fn shell_style_empty_string() {
        assert_eq!(shell_style_tokeniser.line(""), Ok(vec![]));
    }

    #[test]
    fn shell_style_leading_whitespace() {
        let rhs = vec![ "abc".into_string(), "def".into_string() ];
        assert_eq!(shell_style_tokeniser.line("     abc def"), Ok(rhs));
    }

    #[test]
    fn shell_style_trailing_whitespace() {
        let rhs = vec![ "ghi".into_string(), "jkl".into_string() ];
        assert_eq!(shell_style_tokeniser.line("ghi jkl     \n"), Ok(rhs));
    }

    #[test]
    fn shell_style_enqueue_command_double_quotes() {
        let lhs =
            "enqueue file \"C:\\\\Users\\\\Test\\\\Artist - Title.mp3\" 1";
        let rhs = vec![ "enqueue".into_string(),
                        "file".into_string(),
                        "C:\\Users\\Test\\Artist - Title.mp3".into_string(),
                        "1".into_string() ];
        assert_eq!(shell_style_tokeniser.line(lhs), Ok(rhs));
    }

    #[test]
    fn shell_style_enqueue_command_single_quotes() {
        let lhs =
            "enqueue file \'C:\\Users\\Test\\Artist - Title.mp3\' 1";
        let rhs = vec![ "enqueue".into_string(),
                        "file".into_string(),
                        "C:\\Users\\Test\\Artist - Title.mp3".into_string(),
                        "1".into_string() ];
        assert_eq!(shell_style_tokeniser.line(lhs), Ok(rhs));
    }

    #[test]
    fn shell_style_escaped_newline() {
        // Unquoted backslash-escape
        assert_eq!(shell_style_tokeniser.line("abc\\\nde"),
                   Ok(vec![ "abc\nde".into_string() ]));
        // Double-quoted backslash-escape
        assert_eq!(shell_style_tokeniser.line("\"abc\\\nde\""),
                   Ok(vec![ "abc\nde".into_string() ]));
        // Double-quoted implicit-escape
        assert_eq!(shell_style_tokeniser.line("\"abc\nde\""),
                   Ok(vec![ "abc\nde".into_string() ]));
        // Single-quoted implicit-escape
        assert_eq!(shell_style_tokeniser.line("\'abc\nde\'"),
                   Ok(vec![ "abc\nde".into_string() ]));
        // But single-quoted backslash-escape shouldn't work
        assert_eq!(shell_style_tokeniser.line("\'abc\\\nde\'"),
                   Ok(vec![ "abc\\\nde".into_string() ]));
    }
}
