use std::fs;
use std::error::Error;
use bincode::config::Options;
use solana_program::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Signature, signer::Signer, signer::keypair::read_keypair_file, transaction};

const VOTE_TXN_RAW: &str = "./examples/vote100";
const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

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
    let rpc_client = RpcClient::new(ENTRYPOINT);
    let voter = read_keypair_file(IDENTITY).unwrap();
    let vote_txn_raw: Vec<u8> = fs::read(VOTE_TXN_RAW).unwrap();
    println!("Vote transaction size={}", vote_txn_raw.len());
    let vote_txn : Option<transaction::VersionedTransaction> = bincode::options()
        .with_limit(1232)
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .deserialize_from(&vote_txn_raw[..])
        .unwrap();
    println!("Vote transaction: {:?}", vote_txn);

    println!("Airdropping to account {:?}", voter.pubkey());
    if let Ok(airdrop_signature) = request_air_drop(&rpc_client, &voter.pubkey(), 1.0) {
        println!("Airdrop finished! Signature: {:?}", airdrop_signature);

   } else {
        println!("Airdrop failed!");
    }
}
