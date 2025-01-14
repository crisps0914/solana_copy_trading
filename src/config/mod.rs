use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub solana: SolanaConfig,
    pub trading: TradingConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub command_prefix: String,
    pub message_timeout: u64,
    pub max_subscribers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub confirmation_blocks: u64,
    pub commitment: String,
    pub max_retries: u32,
    pub retry_delay: u64,
    pub raydium_program_id: String,
    pub jupiter_program_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub min_trade_amount: f64,
    pub max_trade_amount: f64,
    pub slippage_tolerance: f64,
    pub default_gas_buffer: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub max_subscriptions_per_user: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            telegram: TelegramConfig {
                command_prefix: "/".to_string(),
                message_timeout: 5000,
                max_subscribers: 1000,
            },
            solana: SolanaConfig {
                confirmation_blocks: 1,
                commitment: "confirmed".to_string(),
                max_retries: 3,
                retry_delay: 1000,
                raydium_program_id: "5quBzjrh1rKB7oRhxbEr7Tx6t8AH7z3maXJJu44HacKL".to_string(),
                jupiter_program_id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
            },
            trading: TradingConfig {
                min_trade_amount: 0.1,
                max_trade_amount: 10.0,
                slippage_tolerance: 0.5,
                default_gas_buffer: 0.01,
            },
            rate_limit: RateLimitConfig {
                max_requests_per_minute: 60,
                max_subscriptions_per_user: 5,
            },
        }
    }
}

pub type SharedConfig = Arc<Config>;