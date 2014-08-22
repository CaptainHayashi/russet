//! The LineTokeniser trait and implementations.
#![experimental]

use builders::types::StockTokeniser;
use tokeniser::Error;


/// A trait allowing simple by-line usage of Tokeniser builders.
///
/// This trait will probably disappear at some point, as adding traits on
/// functions is slightly weird.
pub trait LineTokeniser {
    fn line(self, ln: &str) -> Result<Vec<String>, Error>;
}

impl LineTokeniser for fn() -> StockTokeniser {
    fn line(self, ln: &str) -> Result<Vec<String>, Error> {
        self().add_line(ln).into_strings()
    }
}
