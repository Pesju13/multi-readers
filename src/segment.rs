use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub(crate) struct Segment<R> {
    inner: R,
    len: Option<u64>,
}

impl<R> Segment<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self { inner, len: None }
    }

    pub(crate) fn get_ref(&self) -> &R {
        &self.inner
    }

    pub(crate) fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for Segment<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for Segment<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<R: Seek> Segment<R> {
    pub(crate) fn len(&mut self) -> std::io::Result<u64> {
        if let Some(len) = self.len {
            return Ok(len);
        }

        let position = self.inner.stream_position()?;
        let len = self.inner.seek(SeekFrom::End(0))?;
        self.inner.seek(SeekFrom::Start(position))?;
        self.len = Some(len);
        Ok(len)
    }
}

#[cfg(feature = "tokio")]
impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for Segment<R> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}
