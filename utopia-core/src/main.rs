mod modules;
mod eventloop;
pub mod frontend;
use eventloop::EventLoop;

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let mut mods = modules::ModuleCore::new()?;
    mods.get_modules();
    let (thread_futures, receiver) = mods.spawn_modules();
    let mut evl = EventLoop::new(thread_futures, receiver);
    evl.run().await;
    Ok(())
}
