use futures::{stream::{FuturesUnordered, StreamExt, SelectAll, select_all}, channel::mpsc, FutureExt};
use tokio::{task::JoinHandle, net::UnixListener, signal::unix::{signal, SignalKind}};
use utopia_module::props;

pub struct EventLoop<'a> {
    thread_futures: FuturesUnordered<JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>>,
    receivers: SelectAll<&'a mut mpsc::UnboundedReceiver<props::ModuleCommands>>
}

// TODO: The usage of lifetimes seems very wierd. Please check if this is accepeble and if it is give it a more constructive name :)
impl <'a> EventLoop<'a> {
    pub fn new(thread_futures: FuturesUnordered<JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>>, channels: std::collections::hash_map::ValuesMut<'a, &'static str, crate::modules::modules::IModule>) -> Self {
        EventLoop {
            thread_futures,
            receivers: select_all(channels.map(|v| v.recv.as_mut().expect("A module had no channel")))
        }
    }
    pub async fn run(&mut self) {
        let listener = UnixListener::bind(format!("{}/utopia.sock", std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR was not set"))).expect("Could not open socket");
        let mut exit_signal = signal(SignalKind::quit()).expect("Failure creating SIGQUIT stream. Are you on Unix?");
        loop {
            futures::select! {
                fe_conn = listener.accept().fuse() => {
                    match fe_conn {
                        Ok((_stream, addr)) => println!("Connection from: {:?}", addr),
                        Err(e) => eprintln!("Error: frontend could not connect to core: {}", e)
                    }
                }
                msg = self.receivers.next() => {
                    match msg {
                        Some(cmd) => {
                            println!("Command: {:?}", cmd);
                        },
                        None => eprintln!("Communication channel of a module died")
                    }
                }
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
