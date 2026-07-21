mod common;

use std::io::{Cursor, Read, Seek, SeekFrom};

use common::FlakySeek;
use multi_readers::MultiReader;

fn seekable_reader() -> MultiReader<Cursor<&'static [u8]>> {
    MultiReader::new([
        Cursor::new(&b"abc"[..]),
        Cursor::new(&b"defg"[..]),
        Cursor::new(&b"hij"[..]),
    ])
}

#[test]
fn seek_from_start_can_cross_reader_boundaries() {
    let mut combined = seekable_reader();
    let mut output = [0; 4];

    assert_eq!(combined.seek(SeekFrom::Start(2)).unwrap(), 2);
    combined.read_exact(&mut output).unwrap();

    assert_eq!(&output, b"cdef");
}

#[test]
fn seek_to_an_exact_boundary_reads_the_next_reader() {
    let mut combined = seekable_reader();
    let mut output = [0; 3];

    assert_eq!(combined.seek(SeekFrom::Start(3)).unwrap(), 3);
    combined.read_exact(&mut output).unwrap();

    assert_eq!(&output, b"def");
}

#[test]
fn seek_from_end_supports_negative_offsets() {
    let mut combined = seekable_reader();
    let mut output = Vec::new();

    assert_eq!(combined.seek(SeekFrom::End(-3)).unwrap(), 7);
    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"hij");
}

#[test]
fn seek_from_current_uses_the_logical_combined_position() {
    let mut combined = seekable_reader();
    let mut prefix = [0; 5];
    combined.read_exact(&mut prefix).unwrap();

    assert_eq!(combined.seek(SeekFrom::Current(-2)).unwrap(), 3);

    let mut output = [0; 4];
    combined.read_exact(&mut output).unwrap();
    assert_eq!(&output, b"defg");
}

#[test]
fn repeated_seeks_reset_later_readers() {
    let mut combined = seekable_reader();
    let mut output = [0; 3];

    combined.seek(SeekFrom::Start(7)).unwrap();
    combined.read_exact(&mut output).unwrap();
    assert_eq!(&output, b"hij");

    combined.seek(SeekFrom::Start(1)).unwrap();
    combined.read_exact(&mut output).unwrap();
    assert_eq!(&output, b"bcd");
}

#[test]
fn seek_can_rewind_after_reading_to_eof() {
    let mut combined = seekable_reader();
    let mut output = Vec::new();
    combined.read_to_end(&mut output).unwrap();

    assert_eq!(combined.seek(SeekFrom::Start(0)).unwrap(), 0);
    output.clear();
    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"abcdefghij");
}

#[test]
fn seeking_before_the_start_returns_an_error() {
    let mut combined = seekable_reader();

    assert!(combined.seek(SeekFrom::Current(-1)).is_err());
    assert!(combined.seek(SeekFrom::End(-11)).is_err());
}

#[test]
fn seek_from_end_can_move_beyond_eof() {
    let mut combined = seekable_reader();

    assert_eq!(combined.seek(SeekFrom::End(5)).unwrap(), 15);
    assert_eq!(combined.stream_position().unwrap(), 15);
}

#[test]
fn an_empty_collection_still_tracks_seek_position() {
    let mut combined = MultiReader::<Cursor<&[u8]>>::default();

    assert_eq!(combined.seek(SeekFrom::Start(8)).unwrap(), 8);
    assert_eq!(combined.stream_position().unwrap(), 8);
}

#[test]
fn a_length_error_does_not_poison_the_cache() {
    let readers = [
        FlakySeek::new(b"ab".to_vec(), false),
        FlakySeek::new(b"cde".to_vec(), true),
    ];
    let mut combined = MultiReader::new(readers);

    assert!(combined.seek(SeekFrom::End(0)).is_err());
    assert_eq!(combined.seek(SeekFrom::End(0)).unwrap(), 5);
}

#[test]
fn total_len_preserves_the_current_read_position() {
    let mut combined = seekable_reader();
    let mut prefix = [0; 2];
    combined.read_exact(&mut prefix).unwrap();

    assert_eq!(combined.total_len().unwrap(), 10);
    assert_eq!(combined.position(), 2);

    let mut next = [0; 3];
    combined.read_exact(&mut next).unwrap();
    assert_eq!(&next, b"cde");
}

#[test]
fn seek_can_return_from_a_position_beyond_eof() {
    let mut combined = seekable_reader();
    let mut output = Vec::new();

    assert_eq!(combined.seek(SeekFrom::Start(20)).unwrap(), 20);
    assert_eq!(combined.read_to_end(&mut output).unwrap(), 0);
    assert_eq!(combined.seek(SeekFrom::Current(-15)).unwrap(), 5);
    combined.read_to_end(&mut output).unwrap();

    assert_eq!(output, b"fghij");
}
