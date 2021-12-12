use structopt::StructOpt;
use anyhow::Result;
use realtps_common::{Block, Chain, JsonDb, Db};

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    ReadBlock { number: u64 },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    println!("opt: {:?}", &opt);

    let test_block = Block {
        chain: Chain::Ethereum,
        block_number: 123,
        timestamp: 333, 
        num_txs: 222,
        hash: "hash".to_string(),
        parent_hash: "parent_hash".to_string(),
    };
    let json_db = Box::new(JsonDb);
    
    json_db.store_block(test_block)?;
    Ok(())
}
