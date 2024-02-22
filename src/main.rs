use std::{fs::File, io::Write, sync::Arc};

use anyhow::Result;
use ethers::{
    providers::{Http, Middleware, Provider, ProviderExt},
    types::{Address, Block, Transaction},
    utils::hex::ToHex,
};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    //连接https rpc
    let client = Arc::new(Provider::<Http>::connect("http://localhost:8545").await);

    let from_block = 19272723_u64;

    // let leatest_block = 19272733_u64;
    let leatest_block = client.get_block_number().await?.as_u64();

    let output_file = "output.txt";

    let mut file = std::fs::File::create(output_file)?;

    let mut tasks = Vec::new();

    for i in from_block..leatest_block {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let block = client.get_block_with_txs(i).await?;
            match block {
                Some(block) => Ok(block),
                None => Err(anyhow::anyhow!("block not found")),
            }
        }));

        if tasks.len() >= 50 {
            process_joins(&mut file, &mut tasks).await?;
            tasks.clear();
        }

        println!("block: {}", i);
    }

    process_joins(&mut file, &mut tasks).await?;
    println!("done");
    Ok(())
}

async fn process_joins(
    file: &mut File,
    joins: &mut Vec<JoinHandle<Result<Block<Transaction>>>>,
) -> Result<()> {
    for join in joins {
        let block = join.await??;
        process_block_with_txs(file, block).await;
    }

    Ok(())
}

async fn process_block_with_txs(file: &mut File, block: Block<Transaction>) {
    let mut froms = vec![];
    for tx in block.transactions {
        froms.push(tx.from);
    }
    froms.sort();
    froms.dedup();
    // file 追加写入

    for from in froms {
        file.write(format!("{}\n", fmt_address(from)).as_bytes())
            .unwrap();
    }
}

pub fn fmt_address(address: Address) -> String {
    format!("0x{}", address.as_bytes().encode_hex::<String>())
}
