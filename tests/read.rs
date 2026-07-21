mod common;

use std::io::{self, Cursor, Read};

use common::{ChunkedReader, InterruptOnce};
use multi_readers::MultiReader;

#[test]
fn concatenates_readers_in_order() {
    let readers = [Cursor::new(&b"hello, "[..]), Cursor::new(&b"world!"[..])];
    let mut combined = MultiReader::new(readers);
    let mut output = String::new();

    combined.read_to_string(&mut output).unwrap();

    assert_eq!(output, "hello, world!");
}

#[test]
fn skips_empty_readers_at_any_position() {
    let readers = [
        Cursor::new(&b""[..]),
        Cursor::new(&b"ab"[..]),
        Cursor::new(&b""[..]),
        Cursor::new(&b"cd"[..]),
        Cursor::new(&b""[..]),
    ];
    let mut combined = MultiReader::new(readers);
    let mut output = Vec::new();

    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcd");
}

#[test]
fn an_empty_collection_is_immediately_at_eof() {
    let mut combined = MultiReader::<Cursor<&[u8]>>::default();
    let mut buf = [0; 8];

    assert_eq!(combined.read(&mut buf).unwrap(), 0);
}

#[test]
fn an_empty_output_buffer_does_not_consume_input() {
    let mut combined = MultiReader::new([Cursor::new(b"abc")]);

    assert_eq!(combined.read(&mut []).unwrap(), 0);

    let mut output = Vec::new();
    combined.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"abc");
}

#[test]
fn a_read_error_does_not_advance_to_the_next_reader() {
    let readers = [
        InterruptOnce::new(Cursor::new(&b"ab"[..])),
        InterruptOnce::ready(Cursor::new(&b"cd"[..])),
    ];
    let mut combined = MultiReader::new(readers);
    let mut buf = [0; 8];

    let error = combined.read(&mut buf).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);

    let mut output = Vec::new();
    combined.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"abcd");
}

#[test]
fn boxed_trait_objects_support_heterogeneous_readers() {
    let readers: Vec<Box<dyn Read>> = vec![
        Box::new(Cursor::new(&b"hello"[..])),
        Box::new(&b", "[..]),
        Box::new(Cursor::new(&b"world"[..])),
    ];
    let mut combined = MultiReader::new(readers);
    let mut output = String::new();

    combined.read_to_string(&mut output).unwrap();

    assert_eq!(output, "hello, world");
}

#[test]
fn try_new_collects_fallible_readers() {
    let readers: [io::Result<_>; 3] = [
        Ok(Cursor::new(&b"ab"[..])),
        Ok(Cursor::new(&b"cd"[..])),
        Ok(Cursor::new(&b"ef"[..])),
    ];
    let mut combined = MultiReader::try_new(readers).unwrap();
    let mut output = String::new();

    combined.read_to_string(&mut output).unwrap();

    assert_eq!(output, "abcdef");
}

#[test]
fn new_accepts_a_flattened_iterator() {
    let readers = [
        Some(Cursor::new(&b"ab"[..])),
        None,
        Some(Cursor::new(&b"cd"[..])),
    ];
    let mut combined = MultiReader::new(readers.into_iter().flatten());
    let mut output = Vec::new();

    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcd");
}

#[test]
fn a_short_read_does_not_advance_to_the_next_reader() {
    let readers = [
        ChunkedReader::new(b"ab".to_vec(), 1),
        ChunkedReader::new(b"cd".to_vec(), 1),
    ];
    let mut combined = MultiReader::new(readers);
    let mut output = Vec::new();

    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcd");
}
