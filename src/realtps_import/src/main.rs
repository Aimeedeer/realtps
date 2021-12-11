use structopt::StructOpt;
use anyhow::Result;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    ReadBlock { number: u32 },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    println!("opt: {:?}", &opt);
    Ok(())
}
