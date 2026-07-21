use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

use crate::MultiReader;

impl<R: AsyncRead + Unpin> AsyncRead for MultiReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        let this = self.get_mut();
        loop {
            let Some(reader) = this.segments.get_mut(this.current) else {
                return Poll::Ready(Ok(()));
            };

            let filled_before = buf.filled().len();
            match Pin::new(reader).poll_read(cx, buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Ready(Ok(())) => {
                    let read = buf.filled().len() - filled_before;
                    if read == 0 {
                        this.current += 1;
                        continue;
                    }

                    this.advance_position(read);
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}
