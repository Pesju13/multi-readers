#![cfg(feature = "tokio")]

mod common;

use std::task::Poll;

use common::asynchronous::{poll_read_once, ChunkedAsyncReader};
use multi_readers::MultiReader;

#[test]
fn async_read_concatenates_readers() {
    let readers = [
        ChunkedAsyncReader::new(b"hello, ".to_vec(), usize::MAX),
        ChunkedAsyncReader::new(b"world!".to_vec(), usize::MAX),
    ];
    let mut combined = MultiReader::new(readers);

    let mut output = Vec::new();
    loop {
        let Poll::Ready(Ok(bytes)) = poll_read_once(&mut combined, 32) else {
            panic!("combined reader did not become ready");
        };
        if bytes.is_empty() {
            break;
        }
        output.extend(bytes);
    }

    assert_eq!(output, b"hello, world!");
}

#[test]
fn async_read_propagates_pending() {
    let mut combined = MultiReader::new([ChunkedAsyncReader::pending_once(b"abc".to_vec())]);

    assert!(matches!(poll_read_once(&mut combined, 8), Poll::Pending));

    let Poll::Ready(Ok(output)) = poll_read_once(&mut combined, 8) else {
        panic!("combined reader did not become ready after being woken");
    };
    assert_eq!(output, b"abc");
}

#[test]
fn async_read_with_an_empty_buffer_is_ready_without_consuming_input() {
    let mut combined = MultiReader::new([ChunkedAsyncReader::new(b"abc".to_vec(), usize::MAX)]);

    let Poll::Ready(Ok(empty)) = poll_read_once(&mut combined, 0) else {
        panic!("an empty read should complete immediately");
    };
    assert!(empty.is_empty());

    let Poll::Ready(Ok(output)) = poll_read_once(&mut combined, 8) else {
        panic!("combined reader did not become ready");
    };
    assert_eq!(output, b"abc");
}

#[test]
fn async_short_read_does_not_advance_to_the_next_reader() {
    let readers = [
        ChunkedAsyncReader::new(b"ab".to_vec(), 1),
        ChunkedAsyncReader::new(b"cd".to_vec(), 1),
    ];
    let mut combined = MultiReader::new(readers);

    let mut output = Vec::new();
    for _ in 0..4 {
        let Poll::Ready(Ok(bytes)) = poll_read_once(&mut combined, 8) else {
            panic!("combined reader did not become ready");
        };
        output.extend(bytes);
    }

    assert_eq!(output, b"abcd");
}

#[test]
fn async_pending_can_resume_with_a_smaller_buffer() {
    let readers = [
        ChunkedAsyncReader::new(b"ab".to_vec(), usize::MAX),
        ChunkedAsyncReader::pending_once(b"c".to_vec()),
    ];
    let mut combined = MultiReader::new(readers);

    let Poll::Ready(Ok(prefix)) = poll_read_once(&mut combined, 8) else {
        panic!("bytes from the first reader should be returned immediately");
    };
    assert_eq!(prefix, b"ab");

    assert!(matches!(poll_read_once(&mut combined, 1), Poll::Pending));

    let Poll::Ready(Ok(output)) = poll_read_once(&mut combined, 1) else {
        panic!("combined reader did not become ready after being woken");
    };
    assert_eq!(output, b"c");
}
