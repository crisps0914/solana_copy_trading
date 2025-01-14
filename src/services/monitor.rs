use crate::services::solana::SolanaService;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use std::sync::Arc;
use teloxide::{types::ChatId, Bot};
use tokio::sync::mpsc;

pub struct MonitorTrader {
    solana_service: Arc<SolanaService>,
    bot: Bot,
    config: Arc<crate::config::Config>,
}

impl MonitorTrader {
    pub fn new(
        solana_service: Arc<SolanaService>,
        bot: Bot,
        config: Arc<crate::config::Config>,
    ) -> Self {
        Self {
            solana_service,
            bot,
            config,
        }
    }

    pub async fn start_monitoring(
        &self,
        address: &str,
        chat_id: ChatId,
    ) -> Result<mpsc::Sender<()>> {
        let pubkey = Pubkey::from_str(address)?;
        let (stop_tx, mut stop_rx) = mpsc::channel(1);
        
        let bot = self.bot.clone();
        let config = self.config.clone();
        let solana_service = self.solana_service.clone();

        tokio::spawn(async move {
            let mut processed_signatures = std::collections::HashSet::new();

            while stop_rx.try_recv().is_err() {
                if let Ok(signatures) = solana_service
                    .client
                    .get_signatures_for_address(&pubkey)
                {
                    for sig_info in signatures {
                        if processed_signatures.contains(&sig_info.signature) {
                            continue;
                        }

                        if let Ok(tx) = solana_service
                            .client
                            .get_transaction(&sig_info.signature, None)
                        {
                            if let Err(e) = Self::process_transaction(&bot, chat_id, &tx).await {
                                log::error!("Error processing transaction: {}", e);
                            }
                        }

                        processed_signatures.insert(sig_info.signature);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(stop_tx)
    }

    async fn process_transaction(
        bot: &Bot,
        chat_id: ChatId,
        tx: &Transaction,
    ) -> Result<()> {
        let message = format!(
            "🔄 New transaction detected\nSignature: {}",
            tx.signatures[0]
        );

        bot.send_message(chat_id, message).await?;
        Ok(())
    }

    async fn process_raydium_transaction(
        &self,
        signature: &str,
        chat_id: ChatId,
    ) -> Result<()> {
        // Implement Raydium-specific transaction processing
        let message = format!(
            "🔄 Raydium Transaction\nSignature: {}\nView on Solscan: https://solscan.io/tx/{}",
            signature, signature
        );

        self.bot.send_message(chat_id, message).await?;
        Ok(())
    }
}