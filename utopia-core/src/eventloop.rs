use futures::{stream::StreamExt, channel::mpsc, FutureExt};
use tokio::signal::unix::{signal, SignalKind};
use crate::{core::{self, InternalCoreFutures}, modules::ModuleCore, frontend::{con, SockStreamMap, socket::UtopiaSocket}, errors};
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
                                    con::FrontendActions::GetGameLibrary => {
                                        let library = con::CoreEvent {
                                            version: String::from("0.0.0"),
                                            action: con::CoreActions::ResponseGameLibrary(self.core.library.to_frontend())
                                        };
                                        if let Err(e) = self.connections.write_stream(uuid, library).await { //TODO: Don't block
                                            eprintln!("Failed writing to FE: {}", e);
                                        }
                                    },
                                    con::FrontendActions::GetGameDetails(guuid) => {
                                        println!("FE {} requested game deteils of {}", uuid, guuid);
                                        // TODO: Implement a converter from Module to FE - or use the same struct
                                        /*match self.core.library.get(uuid) {
                                            Ok(item) => {
                                                let details = con::CoreEvent {
                                                    version: String::from("0.0.0"),
                                                    action: con::CoreActions::ResponseItemDetails(item.details)
                                                };
                                                if let Err(e) = self.connections.write_stream(uuid, details).await { //TODO: Don't block
                                                    eprintln!("Failed writing to FE: {}", e);
                                                }
                                            },
                                            Err(e) => eprintln!("Failed to get library item: {}", e)
                                        }*/
                                    },
                                    con::FrontendActions::GameMethod(method) => {
                                        match method.method {
                                            con::library::LibraryItemRunnerMethods::Run(guuid) => {
                                                match self.core.library.launch_library_item(guuid.clone(), &self.mods.mod_mgr) {
                                                    Ok(()) => println!("Launching game {}", guuid),
                                                    Err(e) => eprintln!("Failed to launch game {}: {}", guuid, e)
                                                }
                                            },
                                            con::library::LibraryItemRunnerMethods::RunRunner(guuid, runner) => {
                                                match self.core.library.launch_library_item_from_provider(guuid.clone(), &self.mods.mod_mgr, runner.clone()) {
                                                    Ok(()) => println!("Launching game {} via {}", guuid, runner),
                                                    Err(e) => eprintln!("Failed to launch game {} via {}: {}", guuid, runner, e)
                                                }
                                            },
                                            con::library::LibraryItemRunnerMethods::ChangeDefaultRunner(guuid, runner) => {
                                                match self.core.library.change_default_provider(guuid.clone(), runner.clone()) {
                                                    Ok(()) => println!("Changing default runner of game {} to {}", guuid, runner),
                                                    Err(e) => eprintln!("Error occured during changing runner: {}", e)
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
                                com::ModuleCommands::AddLibraryItem(item) => {
                                    if let Err(e) = self.core.library.insert(uuid, item, &self.mods.mod_mgr) {
                                        eprintln!("Error adding a single item to library: {}", e);
                                    }
                                },
                                com::ModuleCommands::AddLibraryItemBulk(items) =>{
                                    if let Err(e) = self.core.library.bulk_insert(uuid, items, &self.mods.mod_mgr) {
                                        eprintln!("Error adding a single item to library: {}", e);
                                    }
                                },
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
