use std::{io::prelude::*,
          path::{Path, PathBuf}};

use tokio::net::UnixDatagram;
use tinytemplate::TinyTemplate;

struct UtopiaDatagramSocket {
	path: PathBuf,
	inner: UnixDatagram
}
impl UtopiaDatagramSocket {
	pub fn bind(path: impl AsRef<Path>) -> std::io::Result<Self> {
		let path = path.as_ref().to_owned();
		UnixDatagram::bind(&path).map(|inner| UtopiaDatagramSocket {
			path,
			inner
		})
	}
}
impl Drop for UtopiaDatagramSocket {
	fn drop(&mut self) {
		// There's no way to return a useful error here
		let _ = std::fs::remove_file(&self.path).unwrap();
	}
}

impl std::ops::Deref for UtopiaDatagramSocket {
	type Target = UnixDatagram;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl std::ops::DerefMut for UtopiaDatagramSocket {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

use std::os::unix::io::FromRawFd;

pub async fn spawn(config: &crate::UtopiaDatabaseConfig) -> failure::Fallible<(tokio::process::Child, String)> {
	std::fs::create_dir_all(&config.working_dir)?;
	let mut tt = TinyTemplate::new();
	let mut template_conf =
		std::fs::File::open(&config.tconfig).map_err(|io| crate::errors::FileError::new(config.tconfig.clone(), io))?;
	let mut tstr = String::new();
	template_conf.read_to_string(&mut tstr)?;
	tt.add_template("redis_config", &tstr)?;
	let conf_str = tt.render("redis_config", config)?;

	let tmpfile = std::ffi::CString::new("/tmp/utopiadb.conf~XXXXXX")?;
	let fileptr = tmpfile.into_raw();
	let (mut file, path) = unsafe {
		let fd = libc::mkstemp(fileptr);
		(
			std::fs::File::from_raw_fd(fd),
			std::ffi::CString::from_raw(fileptr).to_string_lossy().into_owned()
		)
	};
	write!(&mut file, "{}", &conf_str)?;

	let listener = UtopiaDatagramSocket::bind(&config.ready_sock)?;
	let db_server = tokio::process::Command::new("/usr/sbin/redis-server")
		.arg(&path)
		.env("NOTIFY_SOCKET", &config.ready_sock)
		.spawn()?;

	loop {
		listener.readable().await?;

		let mut buf = Vec::new();
		match listener.try_recv_buf_from(&mut buf) {
			Ok(_n) => {
				let msg = String::from_utf8(buf)?;
				let split: Vec<&str> = msg.trim().split("=").collect();
				// Waits for message with "READY=1" as content
				if split.len() == 2 && split[0] == "READY" && split[1] == "1" {
					println!("µtopia database is ready!");
					break;
				}
			},
			Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
				continue;
			},
			Err(e) => {
				return Err(failure::Error::from_boxed_compat(Box::new(e)));
			}
		}
	}
	Ok((db_server, path))
}
