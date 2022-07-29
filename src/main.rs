//! Script assumes that no other scripts or clients are generating transactions.

use chrono::prelude::*;
use iroha_client::client::Client;
use iroha_config::client::Configuration;
use iroha_data_model::{
    events::prelude::*,
    prelude::{Account, AccountId, Instruction, RegisterBox},
    IdentifiableBox,
};
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
use tracing::{info, Level, debug, warn};
use tracing_subscriber::FmtSubscriber;

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
    pub accounts: i64,
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

fn main() -> color_eyre::eyre::Result<()> {
    info!("Welcome to the Iroha 2 longevity load script");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to init logging");
    info!("Staring load script");
    let args = Args::from_args();
    let status = Arc::new(RwLock::new(Status::default()));
    let status_clone_1 = Arc::clone(&status);
    let status_clone_2 = Arc::clone(&status);
    let config_file = File::open("config.json").expect("`config.json` not found.");
    info!("Reading config file");
    let cfg: Configuration =
        serde_json::from_reader(config_file).expect("Failed to deserialize configuration.");
    warn!("No status updates are given in the logs. To access that information please use `curl -X GET {} -i", args.address);

    info!("Reading configuration finished");
    debug!("Configuration: {:#?}", cfg);
    let client = Client::new(&cfg)?;
    let client_clone = client.clone();
    info!("Spawning clients");
    thread::Builder::new().name("event_listener".to_owned()).spawn(move || {
        let _e = ExitOnPanic;
        debug!("ExitOnPanic installed");
        let event_filter = FilterBox::Pipeline(PipelineEventFilter::new());
        for event in client
            .listen_for_events(event_filter)
            .expect("Failed to listen for events")
        {
            if let Ok(Event::Pipeline(event)) = event {
                match event.status {
                    PipelineStatus::Validating => {}
                    PipelineStatus::Rejected(_) => {
                        status_clone_2
                            .write()
                            .expect("Failed to lock to write rejection timestamp to status")
                            .latest_rejected_transaction = Some(Utc::now());
                        status_clone_2
                            .write()
                            .expect("Failed to lock to write rejection increment to status")
                            .txs_rejected += 1;
                    }
                    PipelineStatus::Committed => {
                        status_clone_2
                            .write()
                            .expect("Failed to lock to write commit timestamp to status")
                            .latest_committed_transaction = Some(Utc::now());
                        status_clone_2
                            .write()
                            .expect("Failed to lock to write commit increment to status")
                            .txs_committed += 1;
                    }
                }
            } else {
                warn!("TX with unknown status");
                status_clone_2
                    .write()
                    .expect("Failed to lock to write unknown status")
                    .txs_unknown += 1;
            }
        }
    }).expect("Failed to spawn");
    info!("First client thread spawned");
    thread::Builder::new().name("transaction_sender".to_owned()).spawn(move || {
        let _e = ExitOnPanic;
        let mut current_accounts = 0;
        let interval = Duration::from_secs_f64(1_f64 / args.tps);
        info!("Submitting alice clones");
        while current_accounts < args.accounts {
            status_clone_1
                .write()
                .expect("Failed to lock to write latest_sent_at")
                .latest_sent_at = Some(Utc::now());
            status_clone_1
                .write()
                .expect("Failed to lock to increment txs_sent")
                .txs_sent += 1;
            let new_account: AccountId = format!("alice{}@wonderland", current_accounts)
                .parse()
                .expect("Failed to parse `alice` clone");
            if let Ok(_) = client_clone.submit_all(vec![Instruction::Register(RegisterBox::new(
                IdentifiableBox::from(Account::new(new_account, [])),
            ))]) {
                thread::sleep(interval);
                current_accounts += 1;
            } else {
                warn!("Submit failed");
                thread::sleep(Duration::from_secs(1));
            }
        }
        info!("Submitting empty transactions");
        loop {
            status_clone_1
                .write()
                .expect("Failed to lock to write latest empty sent at")
                .latest_sent_at = Some(Utc::now());
            status_clone_1
                .write()
                .expect("Failed to lock to write latest empty incrment")
                .txs_sent += 1;
            client_clone
                .submit_all(vec![])
                .expect("Failed to submit empty ISI");
            thread::sleep(interval);
        }
    }).expect("Failed to spawn");
    info!("Second thread is spawned. Starting server");
    rouille::start_server(args.address, move |_| {
        Response::json(&*status.read().expect("Failed to read json response"))
    });
}
