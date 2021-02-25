use futures::{stream::StreamExt, channel::mpsc, FutureExt};
use tokio::signal::unix::{signal, SignalKind};
use crate::{core::{self, InternalCoreFutures}, modules::ModuleCore, frontend::{SockStreamMap, socket::UtopiaSocket}, errors};
use utopia_common::{module, frontend};
pub struct EventLoop {
    core: core::Core,
    mods: ModuleCore,
    channel: mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>,
    socket: UtopiaSocket,
    connections: SockStreamMap
}

#[macro_export]
macro_rules! result_printer {
    ($res:expr, $msg:expr) => {
        if let Err(e) = $res {
            eprintln!("{}: {}", $msg, e);
        }
    }
}

impl EventLoop {
    pub fn new(mods: ModuleCore, channel: mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>) -> Self {
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
                                    frontend::FrontendActions::GetGameLibrary => {
                                        let library = frontend::CoreEvent::new(frontend::CoreActions::ResponseGameLibrary(self.core.library.to_frontend()));
                                        result_printer!(self.connections.write_stream(uuid, library).await, "Failed writing to FE"); //TODO: Don't block
                                    },
                                    frontend::FrontendActions::GetGameDetails(guuid) => {
                                        println!("FE {} requested game details of {}", uuid, guuid);
                                        match self.core.library.get(guuid) {
                                            Ok(item) => {
                                                let details = frontend::CoreEvent::new(frontend::CoreActions::ResponseItemDetails(item.details.clone()));
                                                result_printer!(self.connections.write_stream(uuid, details).await, "Failed writing to FE"); //TODO: Don't block
                                            },
                                            Err(e) => eprintln!("Failed to get library item: {}", e)
                                        }
                                    },
                                    frontend::FrontendActions::GameMethod(method) => {
                                        match method {
                                            frontend::library::LibraryItemProviderMethods::Run(guuid) =>
                                                result_printer!(self.core.library.launch_library_item(guuid, &self.mods.mod_mgr), "Error running item"),
                                            frontend::library::LibraryItemProviderMethods::RunProvider(guuid, provider) =>
                                                result_printer!(self.core.library.launch_library_item_from_provider(guuid, &self.mods.mod_mgr, provider), "Error running item via provider"),
                                            frontend::library::LibraryItemProviderMethods::ChangeDefaultProvider(guuid, provider) =>
                                                result_printer!(self.core.library.change_default_provider(guuid, provider), "Error changing provider"),
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
                                module::ModuleCommands::Refresh => println!("Module wants to force a FE refresh"),
                                module::ModuleCommands::AddLibraryItem(item) =>
                                    result_printer!(self.core.library.insert(uuid, item, &self.mods.mod_mgr), "Error adding an item to library"),
                                module::ModuleCommands::AddLibraryItemBulk(items) =>
                                    result_printer!(self.core.library.bulk_insert(uuid, items, &self.mods.mod_mgr), "Error adding items to library"),
                                module::ModuleCommands::ItemStatusSignal(sig) => println!("Received LibraryItem Status Signal: {:?}", sig)
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
