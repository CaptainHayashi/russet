//! Builder for the whitespace-split stock tokeniser.
#![experimental]

use std::collections::hashmap::HashMap;

use builders::types::{
    StockEscapeMap,
    StockQuoteMap,
    StockTokeniser
};
use tokeniser::Tokeniser;


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
    let quote_map: StockQuoteMap = HashMap::new();
    let escape_map: StockEscapeMap = HashMap::new();
    Tokeniser::new(quote_map, escape_map)
}


#[cfg(test)]
mod test {
    use super::whitespace_split_tokeniser;
    use line::LineTokeniser;
    use tokeniser::Error;

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
