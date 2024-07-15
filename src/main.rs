use std::{fs, env};
use bincode::config::Options;
use std::collections::VecDeque;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{hash, instruction::{AccountMeta, Instruction}, signer::{keypair::read_keypair_file, Signer}, transaction, vote::{instruction::VoteInstruction, state::{Lockout, VoteStateUpdate}}};

const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ACCOUNT: &str      = "/home/yunzhang/repos/keypairs/fd-vote-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

fn main() {
    let mut vote_txn: transaction::Transaction;
    let rpc_client  = RpcClient::new(ENTRYPOINT);
    let vote_account = read_keypair_file(ACCOUNT).unwrap();
    let voter_identity = read_keypair_file(IDENTITY).unwrap();
    let recent_hash = rpc_client.get_latest_blockhash().unwrap();

    /* Get recent blockhash and sign the vote txn */
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        /* Read a raw vote txn from file */
        let vote_txn_raw: Vec<u8> = fs::read(&args[1]).unwrap();
        vote_txn = bincode::options()
            .with_limit(1232 as u64)
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(&vote_txn_raw)
            .unwrap();
        println!("Vote transaction raw_sz={:?}", vote_txn_raw.len());
        vote_txn.sign(&[voter_identity], recent_hash);
    } else {
        let root = Option::Some(0 as u64);
        let hash = hash::Hash::new_unique();
        let lockout1 = Lockout::new_with_confirmation_count(0, 18);
        let lockout2 = Lockout::new_with_confirmation_count(1, 17);
        let lockout3 = Lockout::new_with_confirmation_count(2, 16);
        let lockouts = VecDeque::from([lockout1, lockout2, lockout3]);
        let vote_instr = VoteInstruction::CompactUpdateVoteState(VoteStateUpdate::new(lockouts, root, hash));
        let instruction = Instruction::new_with_bincode(solana_sdk::vote::program::ID,
                                                       &vote_instr,
                                                       vec![AccountMeta::new(voter_identity.pubkey(), true),
                                                        AccountMeta::new(vote_account.pubkey(), false)]
                                                        );
        vote_txn = transaction::Transaction::new_signed_with_payer(
            &[instruction],
            Some(&voter_identity.pubkey()),
            &[voter_identity],
            recent_hash,
        );
    }
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
