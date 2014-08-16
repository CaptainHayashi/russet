//! The EscapeScheme trait and implementations.
#![experimental]

use std::collections::hashmap::HashMap;


/// An escaping scheme.
///
/// The Tokeniser maps escape leader characters to escape schemes, which
/// configure how the character following the leader is interpreted.
///
/// Russet comes with an implementation of EscapeScheme for SimpleEscapeScheme.
pub trait EscapeScheme {
    /// Attempts to map an escaped character, `chr`, to its literal substitute.
    ///
    /// # Return value
    ///
    /// An Option, which is `Some(x)` when `chr` is a valid escape character
    /// with subsititute `x`, and `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::hashmap::HashMap;
    /// use russet::escape_scheme::EscapeScheme;
    /// use russet::escape_scheme::{ SimpleEscapeScheme, LiteralEscape };
    /// use russet::escape_scheme::c_escapes;
    ///
    /// // Using a LiteralEscape
    /// let le: SimpleEscapeScheme<HashMap<char, char>> = LiteralEscape;
    /// assert_eq!(le.escape('\n'), Some('\n'));
    ///
    /// // Using a MapEscape
    /// assert_eq!(c_escapes().escape('r'), Some('\r'));
    /// ```
    fn escape(&self, chr: char) -> Option<char>;
}


/// An enumeration of simple escape schemes.
#[deriving(Clone)]
pub enum SimpleEscapeScheme<M> {
    /// Any character prefixed by an escape leader is treated as its literal
    /// value.  This is similar to the shell style of character escaping.
    LiteralEscape,

    /// Any character prefixed by an escape leader is looked up in the map,
    /// and the corresponding entry substituted for the escape sequence.
    MapEscape(M)
}

impl<M> EscapeScheme for SimpleEscapeScheme<M> where M: Map<char, char> {
    fn escape(&self, chr: char) -> Option<char> {
        match *self {
            LiteralEscape => Some(chr),
            MapEscape(ref map) => map.find(&chr).map(|c| c.clone())
        }
    }
}


/// A constructor for a C-style escape sequence.
pub fn c_escapes() -> SimpleEscapeScheme<HashMap<char, char>> {
    let map: HashMap<char, char> =
        vec![ ( 'n',  '\n' ),
              ( 'r',  '\r' ),
              ( '\"', '\"' ),
              ( '\'', '\'' ),
              ( '\\', '\\' ),
              ( 't',  '\t' ) ].move_iter().collect();
    MapEscape(map)
}
