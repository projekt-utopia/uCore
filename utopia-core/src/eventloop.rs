use futures::{stream::StreamExt, channel::mpsc, FutureExt};
use tokio::signal::unix::{signal, SignalKind};
use crate::{modules::ModuleCore, frontend::{con, SockStreamMap, socket::UtopiaSocket}};
use crate::core;
use utopia_module::com;
pub struct EventLoop {
    core: core::Core,
    mods: ModuleCore,
    channel: mpsc::UnboundedReceiver<(&'static str, com::ModuleCommands)>,
    socket: UtopiaSocket,
    connections: SockStreamMap
}

impl EventLoop {
    pub fn new(mods: ModuleCore, channel: mpsc::UnboundedReceiver<(&'static str, com::ModuleCommands)>) -> Self {
        EventLoop {
            core: core::Core::new(),
            mods,
            channel,
            socket: UtopiaSocket::bind(format!("{}/utopia.sock", std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR was not set"))).expect("Could not open socket"),
            connections: SockStreamMap::new()
        }
    }
    pub async fn run(&mut self) {
        //let (itx, mut irx) = mpsc::channel::<core::InternalCoreCom>(0xF);
        let mut exit_signal = signal(SignalKind::quit()).expect("Failure creating SIGQUIT stream. Are you on Unix?");
        loop {
            futures::select! {
                // Internal core communication
                com = self.core.internal_futures.select_next_some() => {
                    println!("Internal core future resolved: {:?}", com);
                }
                // New connection on socket
                con = self.socket.next() => {
                    match con.unwrap() {
                        Ok((stream, _addr)) => {
                            match self.connections.insert(stream).await {
                                Ok(()) => (),
                                Err(e) => println!("Error accepting stream: {}", e)
                            };
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
                                    con::FrontendActions::GetGameLibrary => println!("FE {} requested game library", uuid),
                                    con::FrontendActions::GetGameDetails(guuid) => println!("FE {} requested game deteils of {}", uuid, guuid),
                                    con::FrontendActions::GameMethod(method) => {
                                        match method.method {
                                            con::library::LibraryItemRunnerMethods::Run(guuid) => {
                                                match self.core.library.launch_library_item(guuid.clone(), &self.mods.mod_mgr) {
                                                    Ok(()) => println!("Launching game {}", guuid),
                                                    Err(e) => eprintln!("Failed to launch game {}: {}", guuid, e)
                                                }
                                            },
                                            _ => eprintln!("FE {} requested unimplemented method {:?}", uuid, method),
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
                                com::ModuleCommands::Refresh => println!("Module wants to force a FE refresh"),
                                com::ModuleCommands::AddLibraryItem(item) => self.core.library.insert(uuid, item),
                                com::ModuleCommands::AddLibraryItemBulk(items) => self.core.library.bulk_insert(uuid, items),
                                com::ModuleCommands::ItemStatusSignal(sig) => println!("Received LibraryItem Status Signal: {:?}", sig)
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
    }
}
