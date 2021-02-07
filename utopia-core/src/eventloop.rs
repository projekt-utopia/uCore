use futures::{stream::{FuturesUnordered, StreamExt}, channel::mpsc, FutureExt};
use tokio::signal::unix::{signal, SignalKind};
use crate::{modules::ThreadHandle, frontend::{SockStreamMap, socket::UtopiaSocket}};
use utopia_module::props;

pub struct EventLoop {
    thread_futures: FuturesUnordered<ThreadHandle>,
    channel: mpsc::UnboundedReceiver<(&'static str, props::ModuleCommands)>,
    socket: UtopiaSocket,
    connections: SockStreamMap
}

impl EventLoop {
    pub fn new(thread_futures: FuturesUnordered<ThreadHandle>, channel: mpsc::UnboundedReceiver<(&'static str, props::ModuleCommands)>) -> Self {
        EventLoop {
            thread_futures,
            channel,
            socket: UtopiaSocket::bind(format!("{}/utopia.sock", std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR was not set"))).expect("Could not open socket"),
            connections: SockStreamMap::new()
        }
    }
    pub async fn run(&mut self) {
        let mut exit_signal = signal(SignalKind::quit()).expect("Failure creating SIGQUIT stream. Are you on Unix?");
        loop {
            futures::select! {
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
                    println!("Message from FE: {:?}", msg);
                }

                // New message from module
                msg = self.channel.next() => {
                    match msg {
                        Some(cmd) => {
                            println!("Command: {:?}", cmd);
                        },
                        None => eprintln!("Communication channel of a module died")
                    }
                }
                // Module handle died
                death = self.thread_futures.select_next_some() => {
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
