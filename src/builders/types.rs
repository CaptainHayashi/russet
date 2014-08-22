//! Type synonyms used in the specification of the stock tokenisers.
#![experimental]

use std::collections::hashmap::HashMap;

use escape_scheme::SimpleEscapeScheme;
use tokeniser::{ Tokeniser, QuoteMode };


/// A type for quote-maps used by Russet builders.
pub type StockQuoteMap = HashMap<char, ( char, QuoteMode )>;


/// A type for escape schemes used by Russet builders.
pub type StockEscapeScheme = SimpleEscapeScheme<HashMap<char, char>>;


/// A type for escape-maps used by Russet builders.
pub type StockEscapeMap = HashMap<char, StockEscapeScheme>;


/// A type for tokenisers returned by Russet builders.
pub type StockTokeniser =
    Tokeniser<StockQuoteMap, StockEscapeMap, StockEscapeScheme>;
