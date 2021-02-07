// thanks to @Shepmaster: see https://stackoverflow.com/questions/40218416/how-do-i-close-a-unix-socket-in-rust
use std::path::{Path, PathBuf};
use tokio::net::{UnixListener, UnixStream, unix::SocketAddr};
use futures::stream::{Stream, FusedStream};
use std::{pin::Pin, task::{Context, Poll}};

pub struct UtopiaSocket {
    path: PathBuf,
    listener: UnixListener,
}

impl UtopiaSocket {
    pub fn bind(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_owned();
        UnixListener::bind(&path).map(|listener| UtopiaSocket { path, listener })
    }
}

impl Stream for UtopiaSocket {
    type Item = std::io::Result<(UnixStream, SocketAddr)>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);
        match Pin::new(&mut this.listener).poll_accept(cx) {
            Poll::Ready(res) => Poll::Ready(Some(res)),
            Poll::Pending => Poll::Pending
        }
    }
}

impl FusedStream for UtopiaSocket {
   fn is_terminated(&self) -> bool {
        !self.path.exists()
   } 
}

impl Drop for UtopiaSocket {
    fn drop(&mut self) {
        // There's no way to return a useful error here
        let _ = std::fs::remove_file(&self.path).unwrap();
    }
}

impl std::ops::Deref for UtopiaSocket {
    type Target = UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}

impl std::ops::DerefMut for UtopiaSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.listener
    }
}
