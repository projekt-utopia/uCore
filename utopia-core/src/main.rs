mod core;
mod database;
mod errors;
mod eventloop;
pub mod frontend;
mod modules;
use eventloop::EventLoop;

use std::path::PathBuf;
use std::env::var as env_var;

#[derive(serde::Serialize)]
pub struct UtopiaDatabaseConfig {
	pub tconfig: PathBuf,
	pub socket: PathBuf,
	pub inherits: PathBuf,
	pub logfile: PathBuf,
	pub working_dir: PathBuf,
	pub ready_sock: PathBuf,
}

pub struct UtopiaConfiguration {
	pub runtime_dir: PathBuf,
	pub socket: PathBuf,
	pub database: UtopiaDatabaseConfig,
}

impl UtopiaConfiguration {
	pub fn new() -> Self {
		let runtime_dir = env_var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");
		let home_dir = env_var("HOME").unwrap_or(unsafe {
			let passwd: Option<&libc::passwd> = libc::getpwuid(libc::getuid()).as_ref();
			passwd
				.map(|v| std::ffi::CStr::from_ptr(v.pw_dir).to_string_lossy().into_owned())
				.expect("Failed to get home dir, try setting HOME")
		});
		let xdg_data = env_var("XDG_DATA_HOME").unwrap_or(format!("{}/.local/share", home_dir));
		let data_dir = format!("{}/utopia", xdg_data);
		UtopiaConfiguration {
			socket: format!("{}/utopia.sock", runtime_dir).into(),
			database: UtopiaDatabaseConfig {
				tconfig: PathBuf::from("/home/admin/workspace/core/µCore/db.conf.in"),
				socket: format!("{}/utopiadb.sock", runtime_dir).into(),
				inherits: PathBuf::from("/home/admin/workspace/core/µCore/default.conf"),
				logfile: format!("{}/db.log", data_dir).into(),
				working_dir: data_dir.into(),
				ready_sock: format!("{}/_utopiadbctl.dsock", runtime_dir).into(),
			},
			runtime_dir: runtime_dir.into(),
		}
	}
}

#[tokio::main]
async fn main() -> failure::Fallible<()> {
	let config = UtopiaConfiguration::new();
	let (child, tmp_path) = database::spawn(&config.database).await?;
	let database = redis::Client::open(format!("unix://{}", config.database.socket.to_string_lossy()))?;
	let shared_db_connection = std::sync::Arc::new(std::sync::RwLock::new(database.get_connection()?));

	let (mods, receiver) = modules::ModuleCore::new(shared_db_connection.clone())?;
	mods.get_modules();
	//let (thread_futures, receiver) = mods.spawn_modules();
	let mut evl = EventLoop::new(config, mods, receiver, child, (database, shared_db_connection));
	evl.run().await;
	let _ = std::fs::remove_file(tmp_path);
	Ok(())
}
