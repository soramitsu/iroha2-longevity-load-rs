//! Script assumes that no other scripts or clients are generating transactions.

use chrono::prelude::*;
use iroha_client::{client::Client, Configuration};
use iroha_data_model::{events::prelude::*, prelude::{AccountId, Instruction, RegisterBox, Account}, IdentifiableBox};
use rouille::Response;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    process,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use structopt::StructOpt;

struct ExitOnPanic;

impl Drop for ExitOnPanic {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("Dropped while unwinding");
            process::exit(1);
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "iroha2-longevity-load",
    about = "Script to generate load for longevity stand."
)]
struct Args {
    #[structopt(long, default_value = "2")]
    pub tps: f64,
    #[structopt(long, default_value = "127.0.0.1:8084")]
    pub address: String,
    #[structopt(long, default_value = "100")]
    pub accounts: i64
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Status {
    txs_committed: usize,
    txs_rejected: usize,
    txs_sent: usize,
    txs_unknown: usize,
    latest_committed_transaction: Option<DateTime<Utc>>,
    latest_rejected_transaction: Option<DateTime<Utc>>,
    latest_sent_at: Option<DateTime<Utc>>,
}

fn main() -> color_eyre::eyre::Result<()>{
    let args = Args::from_args();
    let status = Arc::new(RwLock::new(Status::default()));
    let status_clone_1 = Arc::clone(&status);
    let status_clone_2 = Arc::clone(&status);
    let config_file = File::open("config.json").expect("`config.json` not found.");
    let cfg: Configuration =
        serde_json::from_reader(config_file).expect("Failed to deserialize configuration.");
    let mut client = Client::new(&cfg)?;
    let mut client_clone = client.clone();
    thread::spawn(move || {
        let _e = ExitOnPanic;
        let event_filter = FilterBox::Pipeline(PipelineEventFilter::new());
        for event in client.listen_for_events(event_filter).unwrap() {
            if let Ok(Event::Pipeline(event)) = event {
                match event.status {
                    PipelineStatus::Validating => {}
                    PipelineStatus::Rejected(_) => {
                        status_clone_2.write().unwrap().latest_rejected_transaction =
                            Some(Utc::now());
                        status_clone_2.write().unwrap().txs_rejected += 1;
                    }
                    PipelineStatus::Committed => {
                        status_clone_2.write().unwrap().latest_committed_transaction =
                            Some(Utc::now());
                        status_clone_2.write().unwrap().txs_committed += 1;
                    }
                }
            } else {
                status_clone_2.write().unwrap().txs_unknown+=1;
            }
        }
    });
    thread::spawn(move || {
        let _e = ExitOnPanic;
        let mut current_accounts = 0;
        let interval = Duration::from_secs_f64(1_f64 / args.tps);
        while current_accounts < args.accounts {
            status_clone_1.write().unwrap().latest_sent_at = Some(Utc::now());
            status_clone_1.write().unwrap().txs_sent +=1;
            let new_account: AccountId = format!("alice{}@wonderland", current_accounts).parse().unwrap();
            if let Ok(_) = client_clone.submit_all(vec![
                Instruction::Register(RegisterBox::new(IdentifiableBox::from(Account::new(new_account, []))))
            ]) {
                thread::sleep(interval);
                current_accounts += 1;
            }
            else {
                println!("Submit failed");
                thread::sleep(Duration::from_secs(1));
            }
        }
        loop {
            status_clone_1.write().unwrap().latest_sent_at = Some(Utc::now());
            status_clone_1.write().unwrap().txs_sent += 1;
            client_clone.submit_all(vec![]).unwrap();
            thread::sleep(interval);
        }
    });
    rouille::start_server(args.address, move |_| {
        Response::json(&*status.read().unwrap())
    })
}
