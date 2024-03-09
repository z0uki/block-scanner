use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use ethers::{
    providers::{Http, Middleware, Provider, ProviderExt},
    types::{Address, H160, U256},
    utils::{hex::ToHexExt, parse_ether},
};
use ethers_core::abi::AbiEncode;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<()> {
    //连接https rpc
    let client = Arc::new(Provider::<Http>::connect("http://localhost:8545").await);

    let from_block = 19380000_u64;

    let leatest_block = client.get_block_number().await?.as_u64();

    let semaphore = Arc::new(Semaphore::new(100));
    let mut tasks = Vec::new();
    let weth = H160::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
    for i in from_block..leatest_block {
        let semaphore = semaphore.clone();
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let block = client.get_block_with_txs(i).await.unwrap();
            let txs = block.unwrap_or_default().transactions;
            for tx in txs {
                if let Ok(Some(receipt)) = client.get_transaction_receipt(tx.hash).await {
                    receipt.logs.iter().for_each(|log| {
                        if log.address == weth && log.topics.len() == 3 && log.topics[0].as_bytes().encode_hex() == "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef" {

                            let amount = U256::from_big_endian(log.data.to_vec().as_slice());
                            let from = log.topics[1].as_bytes().encode_hex();
                            let to = log.topics[2].as_bytes().encode_hex();
                            println!("tx: {} from: {}, to: {}, amount: {}", tx.hash.encode_hex(),from, to, amount);
                            if amount > parse_ether(27.04).unwrap() && amount < parse_ether(27.05).unwrap() {
                                println!("tx: {} from: {}, to: {}, amount: {}", tx.hash().encode_hex(),from, to, amount);
                            }
                        }
                    });
                }
            }
        }));
    }

    futures::future::join_all(tasks).await;
    println!("done");
    Ok(())
}

pub fn fmt_address(address: Address) -> String {
    format!("0x{}", address.as_bytes().encode_hex())
}
