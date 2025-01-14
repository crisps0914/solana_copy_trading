use crate::utils::wallet::WalletManager;
use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use teloxide::types::Message;

pub struct SolanaService {
    pub client: RpcClient,
    wallet: Arc<WalletManager>,
    config: Arc<crate::config::Config>,
}

impl SolanaService {
    pub fn new(config: Arc<crate::config::Config>) -> Result<Self> {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .context("SOLANA_RPC_URL not found in environment")?;
        
        let client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        );

        let private_key = std::env::var("SOLANA_PRIVATE_KEY")
            .context("SOLANA_PRIVATE_KEY not found in environment")?;
        
        let wallet = WalletManager::new_from_base64(&private_key)?;

        Ok(Self {
            client,
            wallet: Arc::new(wallet),
            config,
        })
    }

    pub async fn get_balance(&self, msg: &Message) -> Result<()> {
        let balance = self.client.get_balance(&self.wallet.keypair().pubkey())?;
        let sol_balance = balance as f64 / 1_000_000_000.0;
        
        // Format balance message
        let balance_msg = format!("Current balance: {} SOL", sol_balance);
        
        Ok(())
    }

    pub async fn watch_address(&self, address: &str) -> Result<()> {
        let pubkey = Pubkey::from_str(address)
            .context("Invalid Solana address")?;

        self.client.get_account_with_commitment(
            &pubkey,
            CommitmentConfig::confirmed(),
        )?;

        Ok(())
    }

    pub async fn send_transaction(&self, transaction: Transaction) -> Result<Signature> {
        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }

    pub fn get_keypair(&self) -> &Keypair {
        self.wallet.keypair()
    }
}