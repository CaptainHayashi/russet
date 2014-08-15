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


pub use builders::{
    shell_style_tokeniser,
    whitespace_split_tokeniser
};
pub use tokeniser::{
    Error,
    IgnoreEscapes,
    ParseEscapes,
    QuoteMode,
    Tokeniser
};

pub mod builders;
pub mod tokeniser;
