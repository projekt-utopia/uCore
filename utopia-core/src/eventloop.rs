use crate::{
	core::{self, InternalCoreFutures},
	errors,
	frontend::{socket::UtopiaSocket, SockStreamMap, ev},
	modules::ModuleCore,
};
use futures::{channel::mpsc, stream::StreamExt, FutureExt};
use tokio::signal::unix::{signal, SignalKind};
use utopia_common::{frontend, library, module};
pub struct EventLoop {
	core: core::Core,
	mods: ModuleCore,
	channel: mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>,
	socket: UtopiaSocket,
	connections: SockStreamMap,
	db_pid: u32,
	database: redis::Client,
}

#[macro_export]
macro_rules! result_printer {
	($res:expr, $msg:expr) => {
		if let Err(e) = $res {
			eprintln!("{}: {}", $msg, e);
		}
	};
}
macro_rules! result_printer_resp {
	($self:expr, $res:expr, $resp:expr) => {
		let (msg_uuid, fe_uuid): (Option<String>, &String) = $resp;
		let (res, msg): (Result<_, Box<dyn std::error::Error>>, &str) = $res;
		if let Err(e) = res {
			let resp = frontend::CoreEvent::new(frontend::CoreActions::Error(msg.to_string(), e.to_string()), msg_uuid);
			result_printer!(
				$self.connections.write_stream(&fe_uuid, resp).await,
				"Failed writing to FE"
			);
			eprintln!("{}: {}", msg, e);
		}
	};
}

impl EventLoop {
	pub fn new(
		config: crate::UtopiaConfiguration,
		mods: ModuleCore,
		channel: mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>,
		mut db_process: tokio::process::Child,
		database: (redis::Client, utopia_module::UDb),
	) -> Self {
		let db_pid = db_process.id().expect("Failed getting pid of database service");
		let (database, _shared) = database;
		let core = core::Core::new();
		core.internal_futures.push(tokio::spawn(async move {
			let res = db_process.wait().await;
			InternalCoreFutures::DatabaseProcessDied(res)
		}));

		EventLoop {
			core,
			mods,
			channel,
			socket: UtopiaSocket::bind(config.socket).expect("Could not open socket"),
			connections: SockStreamMap::new(),
			db_pid,
			database,
		}
	}
	pub async fn run(&mut self) {
		let mut exit_signal = signal(SignalKind::quit()).expect("Failure creating SIGQUIT stream. Are you on Unix?");
		loop {
			futures::select! {
				// Internal core communication
				com = self.core.internal_futures.select_next_some() => {
					match com {
						Ok(msg) => {
							match msg {
								InternalCoreFutures::NewFrontendRegistered(name, stream) => {
									if let Err(e) = self.connections.insert(name, stream).await {
										eprintln!("Failed to add stream to StreamMap: {}", e);
									}
								},
								InternalCoreFutures::ProcessDied(pid, status) => {
									if status != 0 {
										eprintln!("Process {} died with an non-zero exit code: {}", pid, status);
									}
									if let Some((module, uuid)) = self.core.running.remove(&pid) {
										result_printer!(self.core.library.get_mut(&uuid).expect("FIX ME").update_state(module.to_string(), core::UpdStateAction::Remove, library::LibraryItemStatus::Running(Some(pid))),
											"Failed remove running state to provider");
										ev::send_updated_item(&self.core.library, &uuid, &mut self.connections).await;
									}
								},
								InternalCoreFutures::DatabaseProcessDied(res) => {
									eprintln!("FATAL ERROR: Database process died unexpectedly: {:?}\nPlease check its log for more information.", res);
									break;
								},
								InternalCoreFutures::Debug => println!("Internal debug future resolved"),
								InternalCoreFutures::Error(e) => eprintln!("Internal future resolved as error: {}", e)
							}
						},
						Err(e) => eprintln!("Internal core futures errored {}", e)
					}
				}
				// New connection on socket
				con = self.socket.next() => {
					match con.unwrap() {
						Ok((stream, _addr)) => {
							self.core.internal_futures.push(tokio::spawn(async move {
								match SockStreamMap::accept_handshake(stream).await {
									Ok((name, stream)) => InternalCoreFutures::NewFrontendRegistered(name, stream),
									Err(e) => {
										eprintln!("FE Handshake failed: {}", e);
										InternalCoreFutures::Error(Box::new(errors::UnkownUtopiaError::new("FE Handshake failed", 0)))
									}
								}
							}))
						},
						Err(e) => eprintln!("Error: frontend could not connect to core: {}", e)
					}
				}
				// New message from frontend over socket
				msg = self.connections.next() => {
					if let Some((uuid, msg)) = msg {
						match msg {
							Ok(msg) => {
								match msg.action {
									frontend::FrontendActions::GetGameLibrary => {
										let library = frontend::CoreEvent::new(frontend::CoreActions::ResponseGameLibrary(self.core.library.to_frontend()), msg.uuid.clone());
										result_printer!(self.connections.write_stream(&uuid, library).await, "Failed writing to FE"); //TODO: Don't block
										//result_printer_resp!(self, (self.connections.write_stream(uuid, library).await, "Failed writing to FE"), (msg.uuid, uuid.clone()));
									},
									frontend::FrontendActions::GetFullGameLibrary => {
										let library = frontend::CoreEvent::new(frontend::CoreActions::ResponseFullGameLibrary(self.core.library.to_full()), msg.uuid.clone());
										result_printer!(self.connections.write_stream(&uuid, library).await, "Failed writing to FE"); //TODO: Don't block
									},
									frontend::FrontendActions::GetGameDetails(guuid) => {
										println!("FE {} requested game details of {}", uuid, guuid);
										match self.core.library.get(&guuid) {
											Ok(item) => {
												let details = frontend::CoreEvent::new(frontend::CoreActions::ResponseItemDetails(item.details.clone()), msg.uuid);
												result_printer!(self.connections.write_stream(&uuid, details).await, "Failed writing to FE"); //TODO: Don't block
											},
											Err(e) => {
												let resp = frontend::CoreEvent::new(frontend::CoreActions::Error(String::from("Failed to get library item"), e.to_string()), msg.uuid);
												result_printer!(self.connections.write_stream(&uuid, resp).await, "Failed writing to FE");
												eprintln!("Failed to get library item: {}", e)
											}
										}
									},
									frontend::FrontendActions::GameMethod(method) => {
										match method {
											frontend::library::LibraryItemProviderMethods::Launch(guuid) => {
												result_printer_resp!(self, (self.core.library.launch_library_item(&guuid, &self.mods.mod_mgr), "Error running item"), (msg.uuid, &uuid));
											},
											frontend::library::LibraryItemProviderMethods::LaunchViaProvider(guuid, provider) => {
												result_printer_resp!(self, (self.core.library.launch_library_item_from_provider(&guuid, &self.mods.mod_mgr, provider), "Error running item via provider"), (msg.uuid, &uuid));
											},
											frontend::library::LibraryItemProviderMethods::ChangeSelectedProvider(guuid, provider) => {
												result_printer_resp!(self, (self.core.library.change_default_provider(&guuid, provider), "Error changing provider"), (msg.uuid.clone(), &uuid));
												match self.core.library.get(&guuid) {
													Ok(item) => {
														let details = frontend::CoreEvent::new(frontend::CoreActions::ResponseGameUpdate(item.to_frontend()), msg.uuid);
														result_printer!(self.connections.broadcast_stream(details).await, "Failed writing to FE");
													},
													Err(_e) => eprintln!("FE {} requested nonexistant item: {}", &uuid, guuid)
												};
											}
											_ => eprintln!("FE {} requested unimplemented method {:?}", uuid, method),
										}
									},
									frontend::FrontendActions::RequestPreferenceDiag(module, item) => {
										if let Ok(imodule) = self.mods.mod_mgr.get_owned(&module) {
											self.core.open_preferences.insert((module, item.clone()), (uuid, msg.uuid));
											result_printer!(imodule.send(utopia_common::module::CoreCommands::RequestPreferenceDiag(item)), "Failed messaging module")
										};
									},
									frontend::FrontendActions::PreferenceDiagUpdate((module, itype), values) => {
										if let Ok(imodule) = self.mods.mod_mgr.get_owned(&module) {
											result_printer!(imodule.send(utopia_common::module::CoreCommands::PreferenceDiagUpdate(itype, values)), "Failed messaging module")
										}
									}
								}
							},
							Err(e) => eprintln!("Received invalid message from {}: {}", uuid, e)
						}
					}
				}

				// New message from module
				msg = self.channel.next() => {
					match msg {
						Some((uuid, cmd)) => {
							match cmd {
								module::ModuleCommands::Refresh => println!("Module wants to force a FE refresh"),
								module::ModuleCommands::AddLibraryItem(item) =>
									result_printer!(self.core.library.insert(uuid, item, &self.mods.mod_mgr), "Error adding an item to library"),
								module::ModuleCommands::AddLibraryItemBulk(items) =>
									result_printer!(self.core.library.bulk_insert(uuid, items, &self.mods.mod_mgr), "Error adding items to library"),
								module::ModuleCommands::ItemStatusSignal(sig) => {
									match sig {
										module::LibraryItemStatusSignals::Launched(guid, pid) => {
											result_printer!(self.core.library.get_mut(&guid).expect("FIX ME").update_state(uuid.to_string(), core::UpdStateAction::Add, library::LibraryItemStatus::Running(Some(pid))),
												"Failed add running state to provider");
											/*let details = frontend::CoreEvent::new(frontend::CoreActions::SignalGameLaunch(guid), None);
											result_printer!(self.connections.broadcast_stream(details).await, "Failed writing to FE"); //TODO: Don't block*/
											self.core.running.insert(pid, (uuid, guid.clone()));
											self.core.internal_futures.push(tokio::spawn(async move {
												let status = unsafe {
													let mut status: libc::c_int = 0;
													libc::waitpid(pid as i32, &mut status, 0);
													status
												};
												InternalCoreFutures::ProcessDied(pid, status)
											}));
											ev::send_updated_item(&self.core.library, &guid, &mut self.connections).await;
										},
										_ => println!("Signal from module: {:?}", sig)
									};
								},
								module::ModuleCommands::PreferenceDiagResponse(itype, diag) => {
									let gt = (uuid.to_string(), itype);
									if let Some((recv, muuid)) = self.core.open_preferences.remove(&gt) {
										let diag = frontend::CoreEvent::new(frontend::CoreActions::PreferenceDiagResponse(gt, diag), muuid);
										result_printer!(self.connections.write_stream(&recv, diag).await, "Failed writing to FE"); //TODO: Don't block
									}
								}
							}
						},
						None => eprintln!("Communication channel of a module died")
					}
				}
				// Module handle died
				death = self.mods.futures.select_next_some() => {
					match death {
						Ok(safe) => {
							match safe.1 {
								Ok(excuse) => eprintln!("The module {} died with an excuse: {:?}", safe.0, excuse),
								Err(e) => eprintln!("The module {} died due to an error: {}", safe.0, e)
							}
						},
						Err(e) => eprintln!("A module crashed: {}", e)
					}
				}
				_ = exit_signal.recv().fuse() => break,
				complete => break
			}
		}
		unsafe {
			// try gracefully killing redis
			libc::kill(self.db_pid as i32, libc::SIGINT);
		}
	}
}
