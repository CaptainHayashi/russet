//! The _Tokeniser_ class.
#![experimental]

use std::char::is_whitespace;


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
pub struct Tokeniser<Q, E> {
    /// The current vector of parsed words.
    vec: Vec<String>,

    /// Whether or not we are currently in a word.
    in_word: bool,

    /// The current closing quote character and quote mode, if any.
    quote: Option<( char, QuoteMode )>,

    /// Whether the tokeniser is currently processing an escape character.
    escaping: bool,

    /// Maps from quote openers to quote closers.
    quote_pairs: Q,

    /// Map from escape characters to their replacements.
    escape_pairs: E,

    /// The character preceding escape characters.
    escape_leader: Option<char>
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
#[deriving(Eq, PartialEq, Show)]
pub enum Error {
    /// A quotation was opened, but not closed.
    UnmatchedQuote,

    /// An escape sequence was started, but not finished.
    UnfinishedEscape
}


impl<Q, E> Tokeniser<Q, E>
    where Q: Map<char, ( char, QuoteMode )>,
          E: Map<char, char>,
          Q: Clone,
          E: Clone,
          Q: Collection {
    /// Creates a new, blank Tokeniser.
    ///
    /// # Arguments
    ///
    /// * `quote_pairs`   - A map, mapping characters that serve as opening
    ///                     quotes to their closing quotes and quote modes.
    /// * `escape_pairs`  - A map, mapping escape characters to the characters
    ///                     they represent.  An empty map is treated as a
    ///                     _shell-style escape_: the
    /// * `escape_leader` - The character, if any, that starts an escape
    ///                     sequence.
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
    ///
    /// let quote_pairs: HashMap<char, ( char, QuoteMode )> =
    ///     vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
    /// let escape_pairs: HashMap<char, char> =
    ///     vec![ ( 'n', '\n' ) ].move_iter().collect();
    /// let tok = Tokeniser::new(quote_pairs, escape_pairs, Some('\\'));
    /// assert_eq!(tok.into_strings(), Ok(vec![]));
    /// ```
    pub fn new(quote_pairs: Q, escape_pairs: E, escape_leader: Option<char>)
               -> Tokeniser<Q, E> {
        Tokeniser {
            vec: vec![ String::new() ],
            in_word: false,
            quote: None,
            escaping: false,
            quote_pairs: quote_pairs,
            escape_pairs: escape_pairs,
            escape_leader: escape_leader
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
    pub fn add_char(self, chr: char) -> Tokeniser<Q, E> {
        let mut new = self.clone();

        match (chr, self) {
            // ESCAPE SEQUENCES
            //   Shell-style escaping (no escapes defined)
            //   -> Echo character
            ( c, Tokeniser { escaping: true, escape_pairs: ref es, .. } )
                if es.is_empty() => new.emit(c),
            // Known escaped character, otherwise
            // -> Escape character
            ( e, Tokeniser { escaping: true, escape_pairs: ref es, .. } )
                if es.contains_key(&e) => {
                let x = es.find(&e).unwrap();

                new.emit(x.clone());
            },

            // ESCAPE LEADER
            //   Escape leader, not in quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser {
                escaping: false,
                quote: None,
                escape_leader: Some(e),
                ..
            } ) if e == c => new.start_escaping(),
            //   Escape leader, in escape-permitting quotes
            //   -> Begin escape (and word if not in one already)
            ( c, Tokeniser {
                escaping: false,
                quote: Some(( _, ParseEscapes )),
                escape_leader: Some(e),
                ..
            } ) if e == c => new.start_escaping(),

            // QUOTE OPENING
            //   Quote opening character, not currently in quoted word
            //   -> Start quoting
            ( q, Tokeniser {
                escaping: false,
                quote: None,
                quote_pairs: ref qs,
                ..
            } ) if qs.contains_key(&q) => {
                new.quote = Some(qs.find(&q).unwrap().clone());
                new.in_word = true;
            },

            // QUOTE CLOSING
            //   Quote closing character, in quoted word, quotes ok
            //   -> Stop quoting
            ( c, Tokeniser { escaping: false, quote: Some(( cc, _ )), .. } )
                if c == cc => {
                new.quote = None;
                new.in_word = true;
            },

            // UNESCAPED WHITESPACE
            //   Unescaped whitespace, while not in a word
            //   -> Ignore
            ( a, Tokeniser { escaping: false, in_word: false, .. } )
                if is_whitespace(a) => (),
            //   Unescaped whitespace, while in a non-quoted word
            //   -> End word
            ( a, Tokeniser { escaping: false, in_word: true, quote: None, .. } )
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
    pub fn add_iterator<I: Iterator<char>>(self, mut it: I) -> Tokeniser<Q, E> {
        it.fold(self, |s, chr| s.add_char(chr))
    }

    /// Feeds a line, `line`, into the Tokeniser.
    ///
    /// # Return value
    ///
    /// A new Tokeniser, representing the state of the Tokeniser after
    /// consuming `line`.
    pub fn add_line(self, line: &str) -> Tokeniser<Q, E> {
        self.add_iterator(line.trim().chars())
    }

    /// Destroys the tokeniser, extracting the string vector.
    ///
    /// # Return value
    ///
    /// A Result, containing the tokenised string vector if the Tokeniser
    /// was in a valid ending state, and an Error otherwise.
    pub fn into_strings(mut self) -> Result<Vec<String>, Error> {
        if self.in_word && self.quote.is_some() {
            Err(UnmatchedQuote)
        } else if self.escaping {
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
        self.escaping = false;
        self.vec.mut_last().mutate(|s| { s.push_char(c); s });
    }

    /// Switches on escape mode.
    /// This automatically sets the Tokeniser to be in a word, if it isn't
    /// already.
    fn start_escaping(&mut self) {
        self.escaping = true;
        self.in_word = true;
    }

    /// Drops the current working string, if it is empty.
    fn drop_empty_current_string(&mut self) {
        if self.vec.last().map(|s| s.is_empty()).unwrap_or(false) {
            self.vec.pop();
        }
    }
}
