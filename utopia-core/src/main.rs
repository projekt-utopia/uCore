mod modules;
mod eventloop;
pub mod frontend;
mod core;
mod errors;
use eventloop::EventLoop;



#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let (mods, receiver) = modules::ModuleCore::new()?;
    mods.get_modules();
    //let (thread_futures, receiver) = mods.spawn_modules();
    let mut evl = EventLoop::new(mods, receiver);
    evl.run().await;
    Ok(())
}
