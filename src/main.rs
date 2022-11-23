//! Script assumes that no other scripts or clients are generating transactions.
mod args;
mod async_client;
mod commands;
mod number;
mod operation;
mod status;
mod value;

use args::RunArgs;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use std::io::{stdout, BufWriter, Write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "iroha2-longevity-load",
    about = "Script to generate load for longevity stand."
)]
enum Args {
    Oneshot(commands::oneshot::Args),
    Daemon(commands::daemon::Args),
}

#[async_trait]
impl RunArgs for Args {
    async fn run<T: Write + Send>(self, writer: &mut BufWriter<T>) -> Result<()> {
        match self {
            Args::Oneshot(comm) => comm.run(writer).await,
            Args::Daemon(comm) => comm.run(writer).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let mut writer = BufWriter::new(stdout());
    args.run(&mut writer).await?;
    Ok(())
}
