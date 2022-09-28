use super::make_instruction_by_operation;
use crate::{
    args::RunArgs,
    async_client::{Client as AsyncClient, SubmitBlockingStatus},
    operation::Operation,
    status::Status,
};
use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr as _};
use iroha_client::client::Client;
use iroha_config::client::Configuration;
use iroha_data_model::prelude::*;
use std::{
    fs::File,
    io::Write,
    str::FromStr,
    sync::{Arc, RwLock},
};
use structopt::StructOpt;
use tokio::task;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, default_value = "100")]
    count: usize,
    #[structopt(long, required = true)]
    operation: Operation,
}

#[async_trait]
impl RunArgs for Args {
    async fn run<T: Write + Send>(self, writer: &mut std::io::BufWriter<T>) -> Result<()> {
        let status = run_oneshot_operation(self.count, self.operation).await?;
        writeln!(writer, "{}", serde_json::to_string_pretty(&status)?)
            .wrap_err("Failed to pretty print a result")?;
        Ok(())
    }
}

async fn run_oneshot_operation(count: usize, operation: Operation) -> Result<Status> {
    let config_file = File::open("config.json").expect("`config.json` not found.");
    let cfg: Configuration =
        serde_json::from_reader(config_file).expect("Failed to deserialize configuration.");
    let client: AsyncClient = AsyncClient::from(Client::new(&cfg)?);
    let status = Arc::new(RwLock::new(Status::default()));
    let mut operation_handles = vec![];
    let alice_id = AccountId::from_str("alice@wonderland").expect("Failed to make Alice id");
    let wonderland_id =
        DomainId::new(Name::from_str("wonderland").expect("Failed to create Wodnerland name"));
    for index in 0..count {
        let status = Arc::clone(&status);
        let alice_id = alice_id.clone();
        let wonderland_id = wonderland_id.clone();
        let client = client.clone();
        let handle = task::spawn(async move {
            status
                .write()
                .expect("Failed to lock to update status")
                .tx_is_sent();
            let instructions =
                make_instruction_by_operation(&operation, alice_id, wonderland_id, index);
            let res = client
                .submit_all_blocking(instructions)
                .await
                .expect("Failed to submit the transaction");
            let mut guard = status.write().expect("Failed to lock to update status");
            match res {
                SubmitBlockingStatus::Committed(_) => {
                    guard.tx_is_committed();
                }
                SubmitBlockingStatus::Rejected(_) => {
                    guard.tx_is_rejected();
                }
                SubmitBlockingStatus::Unknown => guard.tx_is_unknown(),
            };
        });

        operation_handles.push(handle);
    }

    for handle in operation_handles {
        handle.await.expect("Failed to handle a spawned task");
    }

    let status = status
        .read()
        .expect("Failed to lock to read a status")
        .clone();
    Ok(status)
}
