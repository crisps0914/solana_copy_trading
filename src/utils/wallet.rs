use anyhow::{Context, Result};
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    pubkey::Pubkey,
};
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::Arc,
};

pub struct WalletManager {
    keypair: Keypair,
}

impl WalletManager {
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)
            .context("Failed to open wallet file")?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .context("Failed to read wallet file")?;
        
        let keypair_bytes: Vec<u8> = serde_json::from_slice(&contents)
            .context("Failed to parse wallet JSON")?;
        
        let keypair = Keypair::from_bytes(&keypair_bytes)
            .context("Failed to create keypair from bytes")?;
        
        Ok(Self { keypair })
    }

    pub fn new_from_base64(private_key: &str) -> Result<Self> {
        let decoded = base64::decode(private_key)
            .context("Failed to decode base64 private key")?;
            
        let keypair = Keypair::from_bytes(&decoded)
            .context("Failed to create keypair from bytes")?;
            
        Ok(Self { keypair })
    }

    pub fn new_from_base58(secret_key: &str) -> Result<Self> {
        let bytes = bs58::decode(secret_key)
            .into_vec()
            .context("Failed to decode base58 string")?;
            
        let keypair = Keypair::from_bytes(&bytes)
            .context("Failed to create keypair from bytes")?;
            
        Ok(Self { keypair })
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn public_key(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub fn public_key_base58(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    pub fn secret_key_base58(&self) -> String {
        bs58::encode(&self.keypair.secret().to_bytes()).into_string()
    }

    pub async fn get_balance(&self, client: &solana_client::rpc_client::RpcClient) -> Result<f64> {
        let balance = client.get_balance(&self.public_key())?;
        Ok(balance as f64 / 1_000_000_000.0) // Convert lamports to SOL
    }
}

impl Clone for WalletManager {
    fn clone(&self) -> Self {
        Self {
            keypair: Keypair::from_bytes(&self.keypair.to_bytes()).unwrap(),
        }
    }
}