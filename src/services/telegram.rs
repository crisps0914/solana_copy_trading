use crate::config::SharedConfig;
use anyhow::Result;
use std::collections::HashSet;
use teloxide::{prelude::*, types::Message};
use tokio::sync::Mutex;

pub struct TelegramService {
    bot: Bot,
    config: SharedConfig,
    subscribers: Mutex<HashSet<i64>>,
}

impl TelegramService {
    pub fn new(bot: Bot, config: SharedConfig) -> Self {
        Self {
            bot,
            config,
            subscribers: Mutex::new(HashSet::new()),
        }
    }

    pub async fn handle_subscribe(&self, msg: Message) -> Result<()> {
        let user_id = msg.chat.id;
        let mut subscribers = self.subscribers.lock().await;

        if subscribers.len() >= self.config.telegram.max_subscribers {
            self.bot
                .send_message(
                    msg.chat.id,
                    "Maximum number of subscribers reached. Please try again later.",
                )
                .await?;
            return Ok(());
        }

        if subscribers.contains(&user_id) {
            self.bot
                .send_message(msg.chat.id, "You are already subscribed!")
                .await?;
            return Ok(());
        }

        subscribers.insert(user_id);
        self.bot
            .send_message(msg.chat.id, "Successfully subscribed to copy trading!")
            .await?;
        Ok(())
    }

    pub async fn handle_unsubscribe(&self, msg: Message) -> Result<()> {
        let user_id = msg.chat.id;
        let mut subscribers = self.subscribers.lock().await;

        if !subscribers.contains(&user_id) {
            self.bot
                .send_message(msg.chat.id, "You are not subscribed!")
                .await?;
            return Ok(());
        }

        subscribers.remove(&user_id);
        self.bot
            .send_message(msg.chat.id, "Successfully unsubscribed from copy trading!")
            .await?;
        Ok(())
    }

    pub async fn notify_subscribers(&self, message: &str) -> Result<()> {
        let subscribers = self.subscribers.lock().await;
        for &user_id in subscribers.iter() {
            if let Err(e) = self.bot.send_message(ChatId(user_id), message).await {
                log::error!("Failed to notify subscriber {}: {}", user_id, e);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.config.telegram.message_timeout,
            ))
            .await;
        }
        Ok(())
    }
}