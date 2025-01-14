use crate::config::Config;
use crate::services::{MonitorTrader, SolanaService, TelegramService};
use crate::utils::helpers::is_valid_solana_address;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Show this help message")]
    Start,
    #[command(description = "Stop monitoring current address")]
    Stop,
    #[command(description = "Check current monitoring status")]
    Status,
    #[command(description = "Check your wallet balance")]
    Balance,
    #[command(description = "Subscribe to notifications")]
    Subscribe,
    #[command(description = "Unsubscribe from notifications")]
    Unsubscribe,
    #[command(description = "View Raydium transactions")]
    Trader,
}

pub struct CopyTradingBot {
    bot: Bot,
    config: Arc<Config>,
    telegram_service: Arc<TelegramService>,
    solana_service: Arc<SolanaService>,
    monitor_trader: Arc<MonitorTrader>,
    active_monitors: Arc<Mutex<HashMap<i64, tokio::sync::mpsc::Sender<()>>>>,
}

impl CopyTradingBot {
    pub async fn new() -> Result<Self> {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")?;
        let bot = Bot::new(bot_token);
        let config = Arc::new(Config::default());

        let solana_service = Arc::new(SolanaService::new(config.clone())?);
        let telegram_service = Arc::new(TelegramService::new(bot.clone(), config.clone()));
        let monitor_trader = Arc::new(MonitorTrader::new(
            solana_service.clone(),
            bot.clone(),
            config.clone(),
        ));

        Ok(Self {
            bot,
            config,
            telegram_service,
            solana_service,
            monitor_trader,
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start(self) -> Result<()> {
        log::info!("Starting bot...");

        let handler = Update::filter_message()
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(move |msg: Message, bot: Bot, cmd: Command| {
                        let this = self.clone();
                        async move { this.handle_command(msg, cmd).await }
                    }),
            )
            .branch(
                dptree::filter(|msg: Message| async move {
                    msg.text().map(|text| !text.starts_with('/')).unwrap_or(false)
                })
                .endpoint(move |msg: Message| {
                    let this = self.clone();
                    async move { this.handle_address(msg).await }
                }),
            );

        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    async fn handle_command(&self, msg: Message, cmd: Command) -> ResponseResult<()> {
        match cmd {
            Command::Start => {
                self.bot
                    .send_message(
                        msg.chat.id,
                        "Welcome to Solana Copy Trading Bot! 🚀\n\
                         Available commands:\n\
                         /start - Show this help message\n\
                         /stop - Stop monitoring\n\
                         /status - Check status\n\
                         /balance - Check balance\n\
                         /subscribe - Subscribe to notifications\n\
                         /unsubscribe - Unsubscribe from notifications\n\
                         /trader - View Raydium transactions",
                    )
                    .await?;
            }
            Command::Stop => {
                self.stop_monitoring(msg.chat.id).await?;
            }
            Command::Status => {
                self.check_status(msg.chat.id).await?;
            }
            Command::Balance => {
                self.solana_service.get_balance(&msg).await?;
            }
            Command::Subscribe => {
                self.telegram_service.handle_subscribe(msg).await?;
            }
            Command::Unsubscribe => {
                self.telegram_service.handle_unsubscribe(msg).await?;
            }
            Command::Trader => {
                self.bot
                    .send_message(msg.chat.id, "Please enter a Solana address to monitor")
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_address(&self, msg: Message) -> ResponseResult<()> {
        if let Some(address) = msg.text() {
            if is_valid_solana_address(address) {
                self.start_monitoring(msg.chat.id, address).await?;
            } else {
                self.bot
                    .send_message(msg.chat.id, "Invalid Solana address format")
                    .await?;
            }
        }
        Ok(())
    }

    async fn start_monitoring(&self, chat_id: ChatId, address: &str) -> ResponseResult<()> {
        let mut monitors = self.active_monitors.lock().await;

        // Stop existing monitor if any
        if let Some(stop_sender) = monitors.remove(&chat_id.0) {
            let _ = stop_sender.send(()).await;
        }

        // Start new monitoring
        match self.monitor_trader.start_monitoring(address, chat_id).await {
            Ok(stop_sender) => {
                monitors.insert(chat_id.0, stop_sender);
                self.bot
                    .send_message(chat_id, format!("Started monitoring address: {}", address))
                    .await?;
            }
            Err(e) => {
                self.bot
                    .send_message(chat_id, format!("Error starting monitoring: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    async fn stop_monitoring(&self, chat_id: ChatId) -> ResponseResult<()> {
        let mut monitors = self.active_monitors.lock().await;
        if let Some(stop_sender) = monitors.remove(&chat_id.0) {
            let _ = stop_sender.send(()).await;
            self.bot
                .send_message(chat_id, "Stopped monitoring")
                .await?;
        } else {
            self.bot
                .send_message(chat_id, "No active monitoring to stop")
                .await?;
        }
        Ok(())
    }

    async fn check_status(&self, chat_id: ChatId) -> ResponseResult<()> {
        let monitors = self.active_monitors.lock().await;
        let status = if monitors.contains_key(&chat_id.0) {
            "Active monitoring in progress"
        } else {
            "No active monitoring"
        };
        self.bot.send_message(chat_id, status).await?;
        Ok(())
    }
}