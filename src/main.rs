use std::{fs, env};
use bincode::config::Options;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{signer::keypair::read_keypair_file, transaction};

const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run [VOTE_TXN_RAW_FILEPATH]");
    } else {
        /* Read a raw vote txn from file */
        let vote_txn_raw: Vec<u8> = fs::read(&args[1]).unwrap();
        let mut vote_txn: transaction::Transaction = bincode::options()
            .with_limit(1232 as u64)
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(&vote_txn_raw)
            .unwrap();
        println!("Vote transaction raw_sz={:?}", vote_txn_raw.len());

        /* Get recent blockhash and sign the vote txn */
        let rpc_client = RpcClient::new(ENTRYPOINT);
        let voter_identity = read_keypair_file(IDENTITY).unwrap();
        vote_txn.sign(&[voter_identity], rpc_client.get_latest_blockhash().unwrap());
        println!("Vote transaction: {:?}", vote_txn);

        /* Send the vote txn */
        if vote_txn.verify_with_results()[0] {
            println!("Sending the vote txn to {:?}", ENTRYPOINT);
            let config = RpcSendTransactionConfig {
                skip_preflight: true,
                .. RpcSendTransactionConfig::default()
            };
            match  rpc_client.send_transaction_with_config(&vote_txn, config) {
                Ok(sig) => {
                    println!("Finish sending vote txn.");
                    loop {
                        let confirmed = rpc_client.confirm_transaction(&sig).unwrap();
                        if confirmed {
                            break;
                        }
                    }
                    println!("Finish confirming vote txn.");
                }
                Err(err) => { println!("Failed at sending vote txn: {:?}", err); }
            }
        } else {
            println!("Vote transaction verification fails.");
        }
    }
}

