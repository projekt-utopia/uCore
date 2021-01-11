mod modules;
mod eventloop;
use eventloop::EventLoop;

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let mut mods = modules::ModuleCore::new()?;
    mods.get_modules();
    let thread_futures = mods.spawn_modules();
    let mut evl = EventLoop::new(thread_futures, mods.get_mod_channel_receivers());
    evl.run().await;
    Ok(())
}
