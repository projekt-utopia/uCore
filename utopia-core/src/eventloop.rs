use futures::{stream::{FuturesUnordered, StreamExt, SelectAll, select_all}, channel::mpsc};
use tokio::task::JoinHandle;
use utopia_module::props;

pub struct EventLoop<'a> {
    thread_futures: FuturesUnordered<JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>>,
    receivers: SelectAll<&'a mut mpsc::UnboundedReceiver<props::ModuleCommands>>
}

impl <'a> EventLoop<'a> {
    pub fn new(thread_futures: FuturesUnordered<JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>>, channels: std::collections::hash_map::ValuesMut<'a, &'static str, crate::modules::modules::IModule>) -> Self {
        EventLoop {
            thread_futures,
            receivers: select_all(channels.map(|v| v.recv.as_mut().expect("A module had no channel")))
        }
    }
    pub async fn run(&mut self) {
        loop {
            futures::select! {
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
                },
                complete => break
            }
        }
    }
}
