use anyhow::Result;
use nulus::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let app = App::new().await;

    app.run().await;

    Ok(())
}
