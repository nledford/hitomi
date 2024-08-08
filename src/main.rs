use anyhow::Result;

use hitomi::logger;

#[tokio::main]
async fn main() -> Result<()> {
    logger::initialize_logger()?;

    Ok(())
}
