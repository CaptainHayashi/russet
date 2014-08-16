//! # Russet
//!
//! __Russet__ is a simple string tokenising library for Rust.  It supports
//! POSIX shell-style separation of a line of _words_ into a vector of
//! strings.
//!
//! It comes with three example tokenisers, employing simple whitespace
//! splitting, POSIX shell-style and C-style tactics, and allows custom
//! tokenisers to be created by specifying the permitted quotation pairs,
//! escape sequences, and escape sequence leading character.
//!
//! Russet is quite basic; it doesn't implement shell-style variable and
//! command expansion, multiple-character escape sequences (such as C unicode
//! sequences), and the array of available ‘stock’ tokenisers is limited.
//! However, it can likely be extended to include these and more.
#![experimental]

#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;


pub use builders::{
    c_style_tokeniser,
    shell_style_tokeniser,
    whitespace_split_tokeniser
};
pub use escape_scheme::{
    EscapeScheme,
    SimpleEscapeScheme,
    LiteralEscape,
    MapEscape
};
pub use tokeniser::{
    Error,
    IgnoreEscapes,
    ParseEscapes,
    QuoteMode,
    Tokeniser
};

pub mod builders;
pub mod escape_scheme;
pub mod tokeniser;
