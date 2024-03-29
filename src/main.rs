use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use ethers::{
    abi::AbiDecode,
    providers::{Http, Middleware, Provider, ProviderExt},
    types::{Address, H160, H256, U256},
    utils::{format_ether, hex::ToHexExt, parse_ether},
};
use ethers_core::abi::AbiEncode;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<()> {
    //连接https rpc
    let client = Arc::new(Provider::<Http>::connect("http://localhost:8545").await);

    let from_block = 19387000_u64;

    let leatest_block = client.get_block_number().await?.as_u64();

    let semaphore = Arc::new(Semaphore::new(100));
    let mut tasks = Vec::new();
    let weth = H160::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
    let transfer_topic: H256 = AbiDecode::decode_hex(
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    )?;
    let withdrawal_topic: H256 = AbiDecode::decode_hex(
        "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65",
    )?;
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
                        if log.address == weth
                            && log.topics.len() == 2
                            && log.topics[0] == withdrawal_topic
                        {
                            let amount = U256::from_big_endian(log.data.to_vec().as_slice());
                            let dst = fmt_address(H160::from_slice(&log.topics[1][12..]));
                            if amount > parse_ether(27.04).unwrap()
                                && amount < parse_ether(27.05).unwrap()
                            {
                                println!(
                                    "tx: {} to: {}, amount: {}",
                                    tx.hash().encode_hex(),
                                    dst,
                                    format_ether(amount)
                                );
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
