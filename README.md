<p>
  <a href="https://crates.io/crates/multi-readers">
    <img alt="Crate Info" src="https://img.shields.io/crates/v/multi-readers.svg"/>
  </a>
</p>

# multi-readers

Expose any number of readers as one continuous reader.

`MultiReader` consumes its inner readers in order, skips empty readers, and
only advances after an inner reader reports EOF. Seekable readers share one
logical position across the concatenated stream.

## Synchronous reading

```rust
use std::io::{Cursor, Read};
use multi_readers::MultiReader;

let readers = [
    Cursor::new(&b"hello, "[..]),
    Cursor::new(&b"world!"[..]),
];
let mut reader = MultiReader::new(readers);

let mut output = String::new();
reader.read_to_string(&mut output)?;
assert_eq!(output, "hello, world!");
# Ok::<_, std::io::Error>(())
```

Different reader types can be combined with trait objects:

```rust
use std::io::{Cursor, Read};
use multi_readers::MultiReader;

let readers: Vec<Box<dyn Read>> = vec![
    Box::new(Cursor::new(&b"hello"[..])),
    Box::new(&b", "[..]),
    Box::new(Cursor::new(&b"world"[..])),
];
let mut reader = MultiReader::new(readers);
# let mut output = Vec::new();
# reader.read_to_end(&mut output)?;
# assert_eq!(output, b"hello, world");
# Ok::<_, std::io::Error>(())
```

## Seeking

When every inner reader implements `Seek`, `MultiReader` implements `Seek` as
one concatenated stream. Seeking beyond EOF is supported, and seeking before
byte zero returns `InvalidInput`.

Inner readers should be positioned at byte zero when constructing a seekable
`MultiReader`.

## Fallible construction

Use `try_new` when opening the readers can fail. It returns the first error
instead of silently dropping failed inputs.

```no_run
use std::fs::File;
use multi_readers::MultiReader;

let reader = MultiReader::try_new([
    File::open("part-1.bin"),
    File::open("part-2.bin"),
])?;
# Ok::<_, std::io::Error>(())
```

## Tokio

Enable the `tokio` feature to implement `tokio::io::AsyncRead` when every
inner reader implements it:

```toml
multi-readers = { version = "0.7", features = ["tokio"] }
```

The crate does not start a runtime or perform reads concurrently; it preserves
the ordering of the supplied readers.

## Migrating from 0.6

- `MultiReaders` was renamed to `MultiReader`; the old name remains as a
  deprecated type alias.
- `wrap!` and `open!` were removed. Use `MultiReader::new`, `try_new`, or a
  `Vec<Box<dyn Read>>` for heterogeneous readers.
- The `async` feature was renamed to `tokio`.
- Replace `.flatten()` on the adapter with
  `MultiReader::new(values.into_iter().flatten())`.
