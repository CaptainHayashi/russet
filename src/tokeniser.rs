//! The _Tokeniser_ class.
#![experimental]

use std::char::is_whitespace;

use escape_scheme::EscapeScheme;


/// A tokeniser object.
///
/// A Tokeniser can be fed characters from an iterator, string, or individually.
/// It is an _immutable_ object: actions on a Tokeniser consume the Tokeniser,
/// and produce a fresh copy of the Tokeniser.
///
/// At any stage, a Tokeniser can be consumed to produce the vector of words
/// it has read, using the `into_strings` method.  This method may fail if the
/// Tokeniser ended in a bad state (in the middle of a quoted string, or in
/// the middle of an escape sequence).
#[deriving(Clone)]
pub struct Tokeniser<Q, E, S> {
    /// The current vector of parsed words.
    vec: Vec<String>,

    /// The current tokeniser error, if any.
    /// An error ‘poisons’ the tokeniser, causing it to ignore any further
    /// input.
    error: Option<Error>,

    /// Whether or not we are currently in a word.
    in_word: bool,

    /// The current closing quote character and quote mode, if any.
    quote: Option<( char, QuoteMode )>,

    /// The current escape scheme in use, if any.
    escape: Option<S>,

    /// Maps from quote openers to quote closers.
    quote_map: Q,

    /// Map from escape leader characters to their schemes.
    escape_map: E,
}


/// A quote mode.
#[deriving(Clone)]
pub enum QuoteMode {
    /// All characters except the closing character have their literal value.
    /// This is equivalent to single-quoting in POSIX shell.
    IgnoreEscapes,

    /// All characters except the closing character and escape sequences
    /// have their literal value.  This is roughly equivalent to
    /// double-quoting in POSIX shell.
    ParseEscapes
}


/// A tokeniser error.
///
/// A Tokeniser's `into_strings` method can fail with one of the following
/// errors if called while the Tokeniser is in an unfinished state.
#[deriving(Clone, Eq, PartialEq, Show)]
pub enum Error {
    /// A quotation was opened, but not closed.
    UnmatchedQuote,

    /// An escape sequence was started, but not finished.
    UnfinishedEscape,

    /// An unknown escape sequence was encountered.
    BadEscape
}


impl<Q, E, S> Tokeniser<Q, E, S>
    where Q: Map<char, ( char, QuoteMode )>,
          E: Map<char, S>,
          S: EscapeScheme,
          Q: Clone,
          E: Clone,
          S: Clone,
          Q: Collection {
    /// Creates a new, blank Tokeniser.
    ///
    /// # Arguments
    ///
    /// * `quote_map`  - A map, mapping characters that serve as opening quotes
    ///                  to their closing quotes and quote modes.
    /// * `escape_map` - A map, mapping escape leader characters to their escape
    ///                  schemes.  An empty map disables escapes.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, with an empty state.  Attempting to take the
    /// string vector of the Tokeniser yields the empty vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::hashmap::HashMap;
    /// use russet::{ Tokeniser, ParseEscapes, QuoteMode };
    /// use russet::{ MapEscape, SimpleEscapeScheme };
    ///
    /// let quote_map: HashMap<char, ( char, QuoteMode )> =
    ///     vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
    /// let escape_pairs: HashMap<char, char> =
    ///     vec![ ( 'n', '\n' ) ].move_iter().collect();
    /// let escape_map: HashMap<char, SimpleEscapeScheme<HashMap<char, char>>> =
    ///     vec![ ( '\\', MapEscape(escape_pairs) )].move_iter().collect();
    /// let tok = Tokeniser::new(quote_map, escape_map);
    /// assert_eq!(tok.into_strings(), Ok(vec![]));
    /// ```
    pub fn new(quote_map: Q, escape_map: E) -> Tokeniser<Q, E, S> {
        Tokeniser {
            vec: vec![ String::new() ],
            error: None,
            in_word: false,
            quote: None,
            escape: None,
            quote_map: quote_map,
            escape_map: escape_map
        }
    }

    /// Feeds a single character `chr` to a Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `chr`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russet::whitespace_split_tokeniser;
    ///
    /// let tok = whitespace_split_tokeniser();
    /// let tok2 = tok.add_char('a').add_char('b').add_char('c');
    /// assert_eq!(tok2.into_strings(), Ok(vec![ "abc".into_string() ]));
    /// ```
    pub fn add_char(self, chr: char) -> Tokeniser<Q, E, S> {
        let mut new = self.clone();

        match (chr, self) {
            // ERROR
            //   Found an error
            //   -> Ignore input
            ( _, Tokeniser { error: Some(_), .. } ) => (),

            // ESCAPE SEQUENCES
            //   Currently escaping
            //   -> Escape via escape scheme.
            ( c, Tokeniser { escape: Some(s), .. } ) => match s.escape(c) {
                Some(cc) => new.emit(cc),
                None     => { new.error = Some(BadEscape); }
            },

            // ESCAPE LEADER
            //   Escape leader, not in quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser { escape: None,
                             quote: None,
                             escape_map: ref e, .. } ) if e.contains_key(&c) =>
                new.start_escaping(c),
            //   Escape leader, in escape-permitting quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser { escape: None,
                             quote: Some(( _, ParseEscapes )),
                             escape_map: ref e, .. } ) if e.contains_key(&c) =>
                new.start_escaping(c),

            // QUOTE OPENING
            //   Quote opening character, not currently in quoted word
            //   -> Start quoting
            ( c, Tokeniser { escape: None, quote: None, quote_map: ref q, .. } )
                if q.contains_key(&c) => {
                new.quote = Some(q.find(&c).unwrap().clone());
                new.in_word = true;
            },

            // QUOTE CLOSING
            //   Quote closing character, in quoted word, quotes ok
            //   -> Stop quoting
            ( c, Tokeniser { escape: None, quote: Some(( cc, _ )), .. } )
                if c == cc => {
                new.quote = None;
                new.in_word = true;
            },

            // UNESCAPED WHITESPACE
            //   Unescaped whitespace, while not in a word
            //   -> Ignore
            ( a, Tokeniser { escape: None, in_word: false, .. } )
                if is_whitespace(a) => (),
            //   Unescaped whitespace, while in a non-quoted word
            //   -> End word
            ( a, Tokeniser { escape: None, in_word: true, quote: None, .. } )
                if is_whitespace(a) => {
                new.in_word = false;
                new.vec.push(String::new());
            },

            // DEFAULT
            //   Anything else
            //   -> Echo
            ( a, _ ) => new.emit(a)
        }

        new
    }

    /// Feeds an Iterator of chars, `it`, into the Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming the characters in `it`.
    pub fn add_iter<I: Iterator<char>>(self, mut it: I) -> Tokeniser<Q, E, S> {
        it.fold(self, |s, chr| s.add_char(chr))
    }

    /// Feeds a string, `string`, into the Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `string`.
    pub fn add_string(self, string: &str) -> Tokeniser<Q, E, S> {
        self.add_iter(string.chars())
    }

    /// Feeds a line, `line`, into the Tokeniser.
    /// This differs from `add_str` in that the line is whitespace-trimmed
    /// before adding.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `line`.
    pub fn add_line(self, line: &str) -> Tokeniser<Q, E, S> {
        self.add_string(line.trim())
    }

    /// Destroys the tokeniser, extracting the string vector.
    ///
    /// # Return value
    ///
    /// A Result, containing the tokenised string vector if the Tokeniser
    /// was in a valid ending state, and an Error otherwise.
    pub fn into_strings(mut self) -> Result<Vec<String>, Error> {
        if self.error.is_some() {
            Err(self.error.unwrap())
        } else if self.in_word && self.quote.is_some() {
            Err(UnmatchedQuote)
        } else if self.escape.is_some() {
            Err(UnfinishedEscape)
        } else {
            self.drop_empty_current_string();
            Ok(self.vec)
        }
    }

    /// Adds a character into a Tokeniser's current string.
    /// This automatically sets the Tokeniser's state to be in a word,
    /// and clears any escape sequence flag.
    fn emit(&mut self, c: char) {
        self.in_word = true;
        self.escape = None;
        self.vec.mut_last().mutate(|s| { s.push_char(c); s });
    }

    /// Switches on escape mode.
    /// This automatically sets the Tokeniser to be in a word, if it isn't
    /// already.
    fn start_escaping(&mut self, c: char) {
        self.escape = self.escape_map.find(&c).map(|a| a.clone());
        self.in_word = true;
    }

    /// Drops the current working string, if it is empty.
    fn drop_empty_current_string(&mut self) {
        if self.vec.last().map(|s| s.is_empty()).unwrap_or(false) {
            self.vec.pop();
        }
    }
}
