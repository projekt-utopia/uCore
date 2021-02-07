pub mod socket;
use futures::stream::{Stream, FusedStream};
use tokio::{net::UnixStream, io::{AsyncRead, AsyncReadExt, ReadBuf}};
use std::pin::Pin;
use futures::task::{Context, Poll};
use std::io;
use std::collections::HashMap;

struct SocketStream {
    inner: UnixStream,
    terminated: bool,
}
impl Stream for SocketStream {
    type Item = io::Result<String>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0; 0xFF];
        let mut reader = ReadBuf::new(&mut buf);
        let stream = Pin::new(&mut self.inner);
        match stream.poll_read(cx, &mut reader) {
            Poll::Ready(Ok(())) => {
                match reader.filled().len() {
                    0 => {
                        self.terminated = true;
                        Poll::Ready(None)
                    },
                    _ => {
                        match std::str::from_utf8(reader.filled()) {
                            Ok(str) => Poll::Ready(Some(Ok(String::from(str)))),
                            Err(_) => {
                                self.terminated = true;
                                Poll::Ready(None)
                            }
                        }
                    }
                }
            },
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending
        }
    }
}

pub struct SockStreamMap {
    inner: HashMap<String, SocketStream>
}
impl SockStreamMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
        }
    }
    pub async fn insert(&mut self, mut stream: UnixStream) -> Result<(), Box<dyn std::error::Error>> {
        stream.readable().await?;
        let mut name: Vec<u8> = vec![0; 0o100];
        let n = stream.read(&mut name).await?;
        name.truncate(n);
        let name = std::str::from_utf8(&name)?;
        // thanks to @APerson and @JayDepp on SO on for this whitespace filter
        // https://stackoverflow.com/a/57063944/10890264
        self.inner.insert(name.chars().filter(|c| !c.is_whitespace()).collect(), SocketStream {inner: stream, terminated: false});
        Ok(())
    }
}
impl Stream for SockStreamMap {
    type Item = (String, io::Result<String>);
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        for (id, stream) in &mut self.inner {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(a)) => {
                    return Poll::Ready(Some((id.to_owned(), a)))
                },
                _ => ()
            }
        }
        Poll::Pending
    }
}
impl FusedStream for SockStreamMap {
    fn is_terminated(&self) -> bool {
        let mut term = true;
        for stream in self.inner.values() {
            if !stream.terminated {
                term = false;
            }
        }
        return term;
    }
}
