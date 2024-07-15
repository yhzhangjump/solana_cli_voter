use std::error::Error;
use solana_program::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::{Keypair, Signature}, signer::Signer};

const ENTRY: &str = "https://api.devnet.solana.com";

pub fn request_air_drop(rpc_client: &RpcClient, pub_key: &Pubkey, amount_sol: f64) -> Result<Signature, Box<dyn Error>> {
    const LAMPORTS_PER_SOL: f64 = 1000000000.0;
    let sig = rpc_client.request_airdrop(&pub_key, (amount_sol * LAMPORTS_PER_SOL) as u64)?;
    loop {
        let confirmed = rpc_client.confirm_transaction(&sig)?;
        if confirmed {
            break;
        }
    }
    Ok(sig)
}

fn main() {
    let rpc_client = RpcClient::new(ENTRY);
    let voter = Keypair::new();

    if let Ok(airdrop_signature) = request_air_drop(&rpc_client, &voter.pubkey(), 1.0) {
        println!("Airdrop finished! Signature: {:?}", airdrop_signature);
    } else {
        println!("Airdrop failed!");
    }
}
