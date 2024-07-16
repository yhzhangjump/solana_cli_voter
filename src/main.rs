use std::{env, fs, str::FromStr};
use bincode::config::Options;
use std::collections::VecDeque;
//use serde::{Serialize, Deserialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{hash, instruction::{AccountMeta, Instruction}, signer::{keypair::read_keypair_file, keypair::Keypair, Signer}, transaction, vote::{instruction::VoteInstruction, state::{Lockout, VoteStateUpdate}}};

const IDENTITY: &str     = "/home/yunzhang/repos/keypairs/fd-identity-keypair.json";
const ACCOUNT: &str      = "/home/yunzhang/repos/keypairs/fd-vote-keypair.json";
const ENTRYPOINT: &str   = "http://localhost:8899";
//const ENTRYPOINT: &str = "https://api.devnet.solana.com";

fn create_vote_txn(
    recent_hash: hash::Hash,
    vote_account: &Keypair,
    validator_identity: &Keypair,
) -> transaction::Transaction {
    /* Initialize fields for a CompactUpdateVoteState instruction */
    let tower_slot_start = 1 as u64;
    let tower_slot_end = 31 as u64;
    let tower_slot_end_hash = hash::Hash::from_str("E6XNgGGqWBV55JBuvaS2edhtejF6HJU1MmCjH6rHdjF1").unwrap();

    let mut lockouts : VecDeque<Lockout> = VecDeque::new();
    for i in tower_slot_start..tower_slot_end + 1 {
        let slot = i;
        let confirmation_count = tower_slot_end as u32 - i as u32 + 1;
        lockouts.push_back( Lockout::new_with_confirmation_count( slot, confirmation_count ) );
    }
    let vote_state_update = VoteStateUpdate {
        root: Option::Some(0 as u64),
        hash: tower_slot_end_hash,
        lockouts: lockouts.clone(),
        timestamp: Option::Some(19950128)
    };
    let vote_instr = VoteInstruction::CompactUpdateVoteState(vote_state_update);
    let instruction = Instruction::new_with_bincode(solana_sdk::vote::program::ID,
                                                    &vote_instr,
                                                    vec![AccountMeta::new(vote_account.pubkey(), false),
                                                         AccountMeta::new(validator_identity.pubkey(), true)]
    );

    /* Create the transaction */
    return transaction::Transaction::new_signed_with_payer(
        &[instruction],
        Some(&validator_identity.pubkey()),
        &[validator_identity],
        recent_hash,
    );
}

fn main() {
    let mut vote_txn: transaction::Transaction;
    let rpc_client  = RpcClient::new(ENTRYPOINT);
    let vote_account = read_keypair_file(ACCOUNT).unwrap();
    let validator_identity = read_keypair_file(IDENTITY).unwrap();
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
        vote_txn.sign(&[validator_identity], recent_hash);
    } else {
        vote_txn = create_vote_txn(recent_hash, &vote_account, &validator_identity);
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
