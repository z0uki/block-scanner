use std::sync::Arc;

use anyhow::Result;
use ethers::{
    providers::{Http, Middleware, Provider, ProviderExt},
    types::Address,
    utils::{hex::ToHex, parse_ether},
};
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<()> {
    //连接https rpc
    let client = Arc::new(Provider::<Http>::connect("http://localhost:8545").await);

    let from_block = 19394000_u64;

    let leatest_block = client.get_block_number().await?.as_u64();

    let value = parse_ether("25").unwrap();

    let semaphore = Arc::new(Semaphore::new(100));
    let mut tasks = Vec::new();

    for i in from_block..leatest_block {
        let semaphore = semaphore.clone();
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let block = client.get_block_with_txs(i).await.unwrap();
            let txs = block.unwrap_or_default().transactions;
            for tx in txs {
                if tx.value == value {
                    println!("tx: {}", tx.hash().to_string());
                }
            }
        }));
    }

    futures::future::join_all(tasks).await;
    println!("done");
    Ok(())
}

pub fn fmt_address(address: Address) -> String {
    format!("0x{}", address.as_bytes().encode_hex::<String>())
}
