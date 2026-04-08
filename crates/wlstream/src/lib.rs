use anchovy::AnchovyStream;
use std::collections::VecDeque;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct WaylandStream {
    inner: AnchovyStream,
}
impl WaylandStream {
    pub fn new(stream: impl Into<UnixStream>) -> std::io::Result<Self> {
        let inner = AnchovyStream::new(stream.into())?;
        Ok(Self { inner })
    }
    pub fn fds(&mut self) -> &mut VecDeque<OwnedFd> {
        &mut self.inner.decode_fds
    }
}

impl AsyncRead for WaylandStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncRead::poll_read(Pin::new(&mut self.get_mut().inner), cx, buf)
    }
}

impl AsyncWrite for WaylandStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.get_mut().inner), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().inner), cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.get_mut().inner), cx)
    }
}
