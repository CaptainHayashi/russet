# Russet

[__Russet__][russet] is a simple string tokenising library for [Rust][rust].  It
allows separation of a line of _words_ into a vector of strings, optionally with
quoting and escape sequences.

It comes with three example tokenisers, employing simple whitespace splitting,
POSIX shell-style and C-style tactics, and allows custom tokenisers to be
created by specifying the permitted quotation pairs, escape sequences, and
escape sequence leading character.

Russet is quite basic; it doesn't implement shell-style variable and command
expansion, multiple-character escape sequences (such as C unicode sequences),
and the array of available ‘stock’ tokenisers is limited. However, it can likely
be extended to include these and more.

Russet is licenced under the [MIT licence](mit).

## Warning

Russet is __unstable__, in both the sense that the API will change, and also
in that the implementation will have bugs, is not 100% set in stone, and is
also likely to change.  Use with caution!

## Requirements

* [Rust][rust] nightly
* [Cargo][cargo] nightly (to build)

## Compilation

Use `cargo build`.  Russet should compile using nightly versions of Rust and
Cargo close to Russet's [last change date][commits]; if there are compilation
failures with newer versions, please file an [issue][issues].

## Usage

### Quickly tokenising lines

Russet comes with builder functions for three simple tokenisers, defined in
`russet::builders` and re-exported in `russet`:

* `whitespace_split_tokeniser` — a simple tokeniser, splitting strings into
  tokens by runs of whitespace;
* `shell_style_tokeniser` — a tokeniser that splits strings into tokens by
  using [POSIX shell][shell] tokenisation rules and escape sequences;
* `c_style_tokeniser` — a tokeniser that splits strings into tokens by
  using [C escape sequences][cescape], and also using double quotes to ignore
  whitespace runs.

Any of these tokeniser builders can be used to split a line into words
simply by calling the `.line()` method on each:

```rust
use russet::whitespace_split_tokeniser;

let words = whitespace_split_tokeniser.line("the quick brown fox");

assert_eq!(words, vec![ "the".into_string(),
                        "quick".into_string(),
                        "brown".into_string(),
                        "fox".into_string() ]);
```

### Tokeniser structs

The three builder functions can also be called directly, returning a
_Tokeniser_ object.  This object supports several methods:

* `add_char` — Pushes a character into the Tokeniser, creating a new
  Tokeniser;
* `add_iter` — Pushes an iterator of characters into the Tokeniser, creating
  a new Tokeniser;
* `add_str` — Pushes a string into the Tokeniser, creating a new Tokeniser;
* `add_line` — As `add_str`, but strips any leading and trailing whitespace;
* `into_strings` — Consumes the Tokeniser, returning a Result that may contain
  a Vec of tokenised strings.

Thus, these two are equivalent:

```rust
whitespace_split_tokeniser.line("the quick brown fox")
whitespace_split_tokeniser().add_line("the quick brown fox").into_strings()
```

### Build your own Tokeniser

A custom _Tokeniser_ can be created using the `Tokeniser::new` function.  This
currently takes:

* A Map from opening quote characters to tuples of their corresponding closing
  quote character and the _quote mode_ (currently `IgnoreEscapes` or
  `ParseEscapes`, similar to POSIX shell's `'single quote'` and "double quote"
  behaviour respectively);
* A Map from escape characters to the literals that should replace them (
  currently, an empty map denotes shell-style escaping in which each character
  stands for itself; this will likely change);
* An _escape leader_ character, which signifies the beginning of an escape
  sequence (usually `\`).  This is an Option; setting it to `None` disables
  escape sequences.

The result is a Tokeniser object that can be used as above.

```rust
// A C-style tokeniser
use russet::Tokeniser;
let quote_pairs: HashMap<char, ( char, QuoteMode )> =
    vec![ ( '\"', ( '\"', ParseEscapes ) ) ].move_iter().collect();
let escape_pairs: HashMap<char, char> =
    vec![ ( 'n',  '\n' ),
          ( 'r',  '\r' ),
          ( '\"', '\"' ),
          ( '\'', '\'' ),
          ( 't',  '\t' ) ].move_iter().collect();
Tokeniser::new(quote_pairs, escape_pairs, Some('\\'))
```

## To do

* Clean up code — Russet was split off another project, and is thus slightly
  messy inside;
* Support delimiters other than whitespace (CSV?);
* Support multiple-character escape sequences;
* Support returning the type of word found (unquoted, quoted with escapes
  ignored, quoted with escapes preserved, etc.), for example to allow shell
  implementations to handle variable/command interpolation properly;
* More tests, bug fixes, and stability.

## Contributing

Contributions are welcome!  Please file [issues][issues] and [pull
requests][prq] at the usual places.

[cargo]:   http://crates.io
[cescape]: https://en.wikipedia.org/wiki/Escape_sequences_in_C#Table_of_escape_sequences
[commits]: https://github.com/CaptainHayashi/russet/commits/master
[issues]:  https://github.com/CaptainHayashi/russet/issues
[mit]:     http://opensource.org/licenses/MIT
[prq]:     https://github.com/CaptainHayashi/russet/pulls
[russet]:  https://github.com/CaptainHayashi/russet
[rust]:    http://www.rust-lang.org
[shell]:   http://pubs.opengroup.org/onlinepubs/009604599/utilities/xcu_chap02.html
