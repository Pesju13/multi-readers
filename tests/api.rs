use std::io::{self, Cursor, Read};

use multi_readers::MultiReader;

#[test]
fn new_accepts_an_iterator_borrowing_local_data() {
    let parts: [&[u8]; 2] = [b"ab", b"cd"];
    let mut combined = MultiReader::new(parts.iter().copied());
    let mut output = Vec::new();

    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcd");
}

#[test]
fn readers_can_be_collected() {
    let mut combined: MultiReader<_> = [Cursor::new(&b"ab"[..]), Cursor::new(&b"cd"[..])]
        .into_iter()
        .collect();
    let mut output = Vec::new();

    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcd");
}

#[test]
fn try_new_returns_the_first_error() {
    let inputs: [io::Result<Cursor<&[u8]>>; 3] = [
        Ok(Cursor::new(b"ab")),
        Err(io::Error::new(io::ErrorKind::NotFound, "missing part")),
        Ok(Cursor::new(b"cd")),
    ];

    let error = MultiReader::try_new(inputs).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

#[test]
fn accessors_expose_readers_without_exposing_internal_state() {
    let combined = MultiReader::new([Cursor::new(&b"ab"[..]), Cursor::new(&b"cd"[..])]);

    assert_eq!(combined.reader_count(), 2);
    assert!(!combined.is_empty());
    assert_eq!(combined.readers().count(), 2);

    let readers = combined.into_readers();
    assert_eq!(readers.len(), 2);
}

#[test]
fn position_tracks_bytes_returned_to_the_caller() {
    let mut combined = MultiReader::new([Cursor::new(&b"abc"[..])]);
    let mut buf = [0; 2];

    assert_eq!(combined.position(), 0);
    assert_eq!(combined.read(&mut buf).unwrap(), 2);
    assert_eq!(combined.position(), 2);
    assert_eq!(combined.read(&mut buf).unwrap(), 1);
    assert_eq!(combined.position(), 3);
}

#[test]
fn default_is_an_empty_reader() {
    let combined = MultiReader::<Cursor<&[u8]>>::default();

    assert!(combined.is_empty());
    assert_eq!(combined.reader_count(), 0);
    assert_eq!(combined.position(), 0);
}
