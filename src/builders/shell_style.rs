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
