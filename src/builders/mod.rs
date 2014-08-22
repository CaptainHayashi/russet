//! Builder functions for common tokeniser configurations.
#![experimental]

pub use builders::c_style::c_style_tokeniser;
pub use builders::whitespace_split::whitespace_split_tokeniser;
pub use builders::shell_style::shell_style_tokeniser;

pub mod c_style;
pub mod whitespace_split;
pub mod shell_style;
pub mod types;
