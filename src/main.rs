use std::{fs, env, error::Error};
use bincode::config::Options;
use solana_program::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Signature, signer::Signer, signer::keypair::read_keypair_file, transaction};

const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run [VOTE_TXN_RAW_FILEPATH]");
    } else {
        let rpc_client = RpcClient::new(ENTRYPOINT);
        let voter = read_keypair_file(IDENTITY).unwrap();
        let vote_txn_raw: Vec<u8> = fs::read(&args[1]).unwrap();
        let vote_txn: transaction::VersionedTransaction = bincode::options()
            .with_limit(1232 as u64)
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(&vote_txn_raw)
            .unwrap();
        println!("Vote transaction: {:?}", vote_txn);

        println!("Airdropping to account {:?}", voter.pubkey());
        if let Ok(airdrop_signature) = request_air_drop(&rpc_client, &voter.pubkey(), 1.0) {
            println!("Airdrop finished! Signature: {:?}", airdrop_signature);
            println!("Sending the vote txn");
            match  rpc_client.send_and_confirm_transaction(&vote_txn) {
                Ok(_) => { println!("Succeeds in sending vote txn."); }
                Err(err) => { println!("Failed at sending vote txn: {:?}", err); }
            }
        } else {
            println!("Airdrop failed!");
        }
    }
}

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
