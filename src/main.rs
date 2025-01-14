mod config;
mod services;
mod utils;

use anyhow::Result;
use dotenv::dotenv;
use log::info;
use services::bot::CopyTradingBot;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init();
    info!("Starting Solana Copy Trading Bot...");

    // Create and start the bot
    let bot = CopyTradingBot::new().await?;
    bot.start().await?;

    Ok(())
}