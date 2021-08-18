pub mod socket;
//pub mod con;
use crate::errors::FrontendNotAvailableError;
use futures::{
	stream::{FusedStream, Stream},
	task::{Context, Poll},
};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::pin::Pin;
use tokio::{
	io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
	net::UnixStream,
};
use utopia_common::frontend;

pub struct SocketStream {
	inner: UnixStream,
	terminated: bool,
}
impl Stream for SocketStream {
	type Item = Result<frontend::FrontendEvent, Box<dyn Error>>;
	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut buf = [0; 0xFF];
		let mut reader = ReadBuf::new(&mut buf);
		let stream = Pin::new(&mut self.inner);
		match stream.poll_read(cx, &mut reader) {
			Poll::Ready(Ok(())) => match reader.filled().len() {
				0 => {
					self.terminated = true;
					Poll::Ready(None)
				}
				_ => match serde_json::from_slice(reader.filled()) {
					Ok(action) => Poll::Ready(Some(Ok(action))),
					Err(e) => Poll::Ready(Some(Err(Box::new(e)))),
				},
			},
			Poll::Ready(Err(e)) => Poll::Ready(Some(Err(Box::new(e)))),
			Poll::Pending => Poll::Pending,
		}
	}
}
impl AsyncWrite for SocketStream {
	fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
		let stream = Pin::new(&mut self.inner);
		stream.poll_write(cx, buf)
	}
	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
		let stream = Pin::new(&mut self.inner);
		stream.poll_flush(cx)
	}
	fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
		let stream = Pin::new(&mut self.inner);
		stream.poll_shutdown(cx)
	}
}

pub struct SockStreamMap {
	inner: HashMap<String, SocketStream>,
}
impl SockStreamMap {
	pub fn new() -> Self {
		Self { inner: HashMap::new() }
	}
	pub async fn accept_handshake(mut stream: UnixStream) -> Result<(String, UnixStream), Box<dyn Error>> {
		stream.readable().await?;
		let mut name: Vec<u8> = vec![0; 0o100];
		let n = stream.read(&mut name).await?;
		name.truncate(n);
		let name = std::str::from_utf8(&name)?;
		// thanks to @APerson and @JayDepp on SO on for this whitespace filter
		// https://stackoverflow.com/a/57063944/10890264
		Ok((name.chars().filter(|c| !c.is_whitespace()).collect(), stream))
	}
	pub async fn insert(&mut self, name: String, stream: UnixStream) -> Result<(), Box<dyn Error>> {
		let success = frontend::CoreEvent {
			version: String::from("0.0.0"),
			uuid: None,
			action: frontend::CoreActions::SignalSuccessHandshake(name.clone()),
		};
		stream.writable().await?;
		stream.try_write(&serde_json::to_vec(&success)?)?;
		self.inner.insert(
			name,
			SocketStream {
				inner: stream,
				terminated: false,
			},
		);
		Ok(())
	}
	pub fn get(&mut self, uuid: &String) -> Result<&mut SocketStream, FrontendNotAvailableError> {
		match self.inner.get_mut(uuid) {
			Some(fe) => Ok(fe),
			None => Err(FrontendNotAvailableError::new(uuid)),
		}
	}
	pub async fn write_stream(&mut self, uuid: &String, msg: frontend::CoreEvent) -> Result<(), Box<dyn Error>> {
		let bytes = serde_json::to_vec(&msg)?;
		self.get(uuid)?.write_all(&bytes).await?;
		Ok(())
	}

	pub async fn broadcast_stream(&mut self, msg: frontend::CoreEvent) -> Result<(), Box<dyn Error>> {
		let bytes = serde_json::to_vec(&msg)?;
		for stream in self.inner.values_mut() {
			stream.write_all(&bytes).await?;
		}
		Ok(())
	}
}
impl Stream for SockStreamMap {
	type Item = (String, Result<frontend::FrontendEvent, Box<dyn Error>>);
	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		self.inner.retain(|k, v| match v.terminated {
			true => {
				println!("SockStream {} removed from Map", k);
				false
			}
			false => true,
		});
		for (id, stream) in &mut self.inner {
			match Pin::new(stream).poll_next(cx) {
				Poll::Ready(Some(a)) => return Poll::Ready(Some((id.to_owned(), a))),
				Poll::Ready(None) => {
					// I'd much rather remove the SocketStream item here, but I can't because I'd have another mutable ref
					/*match self.inner.remove(id) {
						Some(_) => println!("FE Connection {} dropped, removing from Map", id),
						None => eprintln!("FE Connection {} dropped, but it was not in the Map (this should not happen!!!)", id)
					}*/
					println!("FE Connection {} died, removing on next iteration", id);
				}
				_ => (),
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
