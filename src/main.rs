use log::{info, error};
use tokio;
use anyhow::Result;

// Import from separated crates
use falcommand_config::Config;
use falcommand_platform::{create_platform_provider};
use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // ログ初期化
    env_logger::init();
    info!("FalCommand starting...");

    // 設定を読み込み
    let config = match Config::load_default().await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // プラットフォーム固有のプロバイダーを初期化
    let platform_provider = create_platform_provider();

    // アプリケーションを初期化
    let mut app = App::new(config, platform_provider).await?;

    // アプリケーションを実行
    app.run().await?;

    info!("FalCommand shutting down...");
    Ok(())
}

mod app;
