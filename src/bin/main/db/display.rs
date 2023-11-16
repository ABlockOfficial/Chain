use bincode::deserialize;
use colored::*;
use rocksdb::{Options, DB};

use naom::primitives::asset::Asset;
use naom::primitives::block::Block;
use naom::primitives::transaction::Transaction;

/// Lists all assets in the blockchain and outputs them to stdout
///
/// ### Arguments
///
/// * `db_path` - Full path to the database
pub fn list_assets(db_path: String) {
    let db = DB::open_default(db_path.clone()).unwrap();

    let mut iter = db.raw_iterator();
    iter.seek_to_first();

    println!();
    while iter.valid() {
        let key_raw = iter.key().unwrap().to_vec();
        let key = String::from_utf8_lossy(&key_raw);

        if !key.starts_with('g') {
            let block = deserialize::<Block>(iter.value().unwrap()).unwrap();
            let previous_hash = match block.clone().header.previous_hash {
                Some(v) => hex::encode(v),
                None => "Φ".to_string(),
            };

            println!("//###########################//");
            println!();
            println!("{}: {}", "BLOCK".magenta(), hex::encode(key_raw));
            println!("{}: {}", "Version".magenta(), block.header.version);
            println!("{}: {:?}", "Previous Hash".magenta(), previous_hash);
            println!(
                "{}: {}",
                "Merkle Root Hash".magenta(),
                hex::encode(block.header.txs_merkle_root_and_hash)
            );
            println!("{}: {}", "Block Number".magenta(), block.header.b_num);
            println!();

            for i in 0..block.transactions.len() {
                let tx_hash = &block.transactions[i];

                let tx = match db.get(tx_hash.to_string()) {
                    Ok(Some(value)) => deserialize::<Transaction>(&value).unwrap(),
                    Ok(None) => panic!("Transaction not found in blockchain"),
                    Err(e) => panic!("Error retrieving block: {:?}", e),
                };

                println!("{} {}", "TRANSACTION".cyan(), i.to_string().cyan());
                println!("{}: {}", "Version".cyan(), tx.version);
                println!(
                    "{}: {:?}",
                    "DRUID".cyan(),
                    tx.druid_info.as_ref().map(|i| &i.druid)
                );
                println!(
                    "{}: {:?}",
                    "DRUID Participants".cyan(),
                    tx.druid_info.as_ref().map(|i| &i.participants)
                );
                println!(
                    "{}: {:?}",
                    "Expected Trade Asset".cyan(),
                    tx.druid_info
                        .iter()
                        .flat_map(|i| i.expectations.iter())
                        .map(|e| &e.asset)
                        .collect::<Vec<_>>()
                );
                println!();
                println!("//------//");
                println!();
                println!("{}", "INPUTS:".cyan());
                for input in &tx.inputs {
                    println!("{}: {:?}", "Previous Out".green(), input.previous_out);
                    println!("{}: {:?}", "Script".green(), input.script_signature);
                    println!();
                }
                println!("//------//");
                println!();
                println!("{}", "OUTPUTS:".cyan());
                for output in &tx.outputs {
                    let (asset_type, asset) = match &output.value {
                        Asset::Token(v) => ("Token", v.to_string()),
                        Asset::Data(v) => ("Data", String::from_utf8_lossy(&v.data).to_string()),
                        Asset::Item(v) => ("Item", v.to_string()),
                    };

                    let drs_root_hash = match &output.drs_block_hash {
                        Some(v) => v.clone(),
                        None => "Φ".to_string(),
                    };

                    let script_pub_key = match &output.script_public_key {
                        Some(v) => v.clone(),
                        None => "Φ".to_string(),
                    };

                    println!("{}: {:?}", "Asset Type".green(), asset_type);
                    println!("{}: {:?}", "Asset".green(), asset);
                    println!("{}: {:?}", "DRS Root Hash".green(), drs_root_hash);
                    println!("{}: {:?}", "Script with PubKey".green(), script_pub_key);
                    println!();
                }
                println!("//###########################//");
            }
        }

        iter.next();
    }
    println!();
    println!("END OF BLOCKCHAIN");
    println!();

    let _ = DB::destroy(&Options::default(), db_path);
}
