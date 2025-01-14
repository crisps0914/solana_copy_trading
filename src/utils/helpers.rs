use lazy_static::lazy_static;
use regex::Regex;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

lazy_static! {
    static ref SOLANA_ADDRESS_REGEX: Regex = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").unwrap();
}

pub fn is_valid_solana_address(address: &str) -> bool {
    if !SOLANA_ADDRESS_REGEX.is_match(address) {
        return false;
    }

    Pubkey::from_str(address).is_ok()
}

pub async fn sleep(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

pub fn format_sol_amount(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

pub fn format_transaction_url(signature: &str) -> String {
    format!("https://solscan.io/tx/{}", signature)
}

pub fn format_error(error: &anyhow::Error) -> String {
    format!("Error: {}", error)
}

pub fn validate_trade_amount(amount: f64, min: f64, max: f64) -> Result<()> {
    if amount < min || amount > max {
        return Err(anyhow::anyhow!(
            "Trade amount must be between {} and {} SOL",
            min,
            max
        ));
    }
    Ok(())
}

pub fn calculate_slippage(price: f64, slippage_percentage: f64) -> f64 {
    price * (1.0 - (slippage_percentage / 100.0))
}

pub async fn retry_with_backoff<T, F, Fut>(
    mut attempts: u32,
    base_delay: u64,
    f: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = base_delay;
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                attempts -= 1;
                if attempts == 0 {
                    return Err(e);
                }
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}