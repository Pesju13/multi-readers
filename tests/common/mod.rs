#![allow(dead_code)]

use std::io::{self, Cursor, Read, Seek, SeekFrom};

pub struct ChunkedReader {
    inner: Cursor<Vec<u8>>,
    max_chunk: usize,
}

impl ChunkedReader {
    pub fn new(bytes: impl Into<Vec<u8>>, max_chunk: usize) -> Self {
        assert!(max_chunk > 0);
        Self {
            inner: Cursor::new(bytes.into()),
            max_chunk,
        }
    }
}

impl Read for ChunkedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.max_chunk);
        self.inner.read(&mut buf[..len])
    }
}

pub struct InterruptOnce<R> {
    inner: R,
    interrupted: bool,
}

impl<R> InterruptOnce<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            interrupted: true,
        }
    }

    pub fn ready(inner: R) -> Self {
        Self {
            inner,
            interrupted: false,
        }
    }
}

impl<R: Read> Read for InterruptOnce<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.interrupted {
            self.interrupted = false;
            Err(io::Error::new(io::ErrorKind::Interrupted, "try again"))
        } else {
            self.inner.read(buf)
        }
    }
}

pub struct FlakySeek {
    inner: Cursor<Vec<u8>>,
    fail_end_once: bool,
}

impl FlakySeek {
    pub fn new(bytes: impl Into<Vec<u8>>, fail_end_once: bool) -> Self {
        Self {
            inner: Cursor::new(bytes.into()),
            fail_end_once,
        }
    }
}

impl Seek for FlakySeek {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if self.fail_end_once && matches!(pos, SeekFrom::End(0)) {
            self.fail_end_once = false;
            Err(io::Error::other("temporary length failure"))
        } else {
            self.inner.seek(pos)
        }
    }
}

#[cfg(feature = "tokio")]
pub mod asynchronous {
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll, Waker};

    use tokio::io::{AsyncRead, ReadBuf};

    pub struct ChunkedAsyncReader {
        bytes: Vec<u8>,
        position: usize,
        max_chunk: usize,
        pending_once: bool,
    }

    impl ChunkedAsyncReader {
        pub fn new(bytes: impl Into<Vec<u8>>, max_chunk: usize) -> Self {
            assert!(max_chunk > 0);
            Self {
                bytes: bytes.into(),
                position: 0,
                max_chunk,
                pending_once: false,
            }
        }

        pub fn pending_once(bytes: impl Into<Vec<u8>>) -> Self {
            Self {
                bytes: bytes.into(),
                position: 0,
                max_chunk: usize::MAX,
                pending_once: true,
            }
        }
    }

    impl AsyncRead for ChunkedAsyncReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            if self.pending_once {
                self.pending_once = false;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            let remaining = &self.bytes[self.position..];
            let len = remaining.len().min(self.max_chunk).min(buf.remaining());
            buf.put_slice(&remaining[..len]);
            self.position += len;
            Poll::Ready(Ok(()))
        }
    }

    pub fn poll_read_once<R: AsyncRead + Unpin>(
        reader: &mut R,
        capacity: usize,
    ) -> Poll<io::Result<Vec<u8>>> {
        let mut cx = Context::from_waker(Waker::noop());
        let mut storage = vec![0; capacity];
        let mut read_buf = ReadBuf::new(&mut storage);

        match Pin::new(reader).poll_read(&mut cx, &mut read_buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(read_buf.filled().to_vec())),
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}
