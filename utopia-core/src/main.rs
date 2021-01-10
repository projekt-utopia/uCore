mod core;

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let mut core = core::Core::new()?;
    core.get_modules();
    core.spawn_modules().await;
    Ok(())
}
