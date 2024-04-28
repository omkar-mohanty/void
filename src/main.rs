use anyhow::Result;
use nulus::run;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    run().await;

    Ok(())
}
