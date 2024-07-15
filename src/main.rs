use std::{fs, env};
use bincode::config::Options;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signer::{keypair::read_keypair_file, Signer}, transaction};

const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run [VOTE_TXN_RAW_FILEPATH]");
    } else {
        let voter = read_keypair_file(IDENTITY).unwrap();
        let vote_txn_raw: Vec<u8> = fs::read(&args[1]).unwrap();
        let vote_txn: transaction::VersionedTransaction = bincode::options()
            .with_limit(1232 as u64)
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(&vote_txn_raw)
            .unwrap();
        println!("Voter pubkey: {:?}", voter.pubkey());
        println!("Vote transaction: {:?}", vote_txn);

        if vote_txn.verify_with_results()[0] {
            println!("Sending the vote txn");
            let rpc_client = RpcClient::new(ENTRYPOINT);
            match  rpc_client.send_and_confirm_transaction(&vote_txn) {
                Ok(_) => { println!("Succeeds in sending vote txn."); }
                Err(err) => { println!("Failed at sending vote txn: {:?}", err); }
            }
        } else {
            println!("Vote transaction verification fails.");
        }
    }
}

