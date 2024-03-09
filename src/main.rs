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
    let deposit_topic: H256 = AbiDecode::decode_hex(
        "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c",
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
                            && log.topics.len() == 3
                            && log.topics[0] == transfer_topic
                        {
                            let amount = U256::from_big_endian(log.data.to_vec().as_slice());
                            let from = fmt_address(H160::from_slice(&log.topics[1][12..]));
                            let to = fmt_address(H160::from_slice(&log.topics[2][12..]));
                            if amount > parse_ether(27.04).unwrap()
                                && amount < parse_ether(27.1).unwrap()
                            {
                                println!(
                                    "tx: {} from: {}, to: {}, amount: {}",
                                    tx.hash().encode_hex(),
                                    from,
                                    to,
                                    format_ether(amount)
                                );
                            }
                        } else if log.address == weth
                            && log.topics.len() == 2
                            && log.topics[0] == deposit_topic
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
