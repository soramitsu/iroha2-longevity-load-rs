//! Script assumes that no other scripts or clients are generating transactions.
mod status;

use crate::status::Status;
use iroha_client::client::Client;
use iroha_config::client::Configuration;
use iroha_crypto::prelude::*;
use iroha_data_model::prelude::*;
use iroha_primitives::fixed::Fixed;
use rouille::Response;
use std::{
    collections::HashMap,
    fs::File,
    process,
    str::FromStr,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use strum_macros::EnumString;
use tracing::{debug, info, warn, Level};
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
    pub tps: usize,
    #[structopt(long, default_value = "127.0.0.1:8084")]
    pub address: String,
    #[structopt(long, default_value = "100")]
    pub count: usize,
    #[structopt(long)]
    pub operation: Vec<Operation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
#[allow(clippy::enum_variant_names)]
enum Operation {
    RegisterAccount,
    RegisterDomain,
    RegisterAssetQuantity,
    RegisterAssetBigQuantity,
    RegisterAssetFixed,
    RegisterAssetStore,
}

fn main() -> color_eyre::eyre::Result<()> {
    info!("Welcome to the Iroha 2 longevity load script");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to init logging");
    info!("Staring load script");
    let args = Args::from_args();
    let operations = args
        .operation
        .into_iter()
        .fold(HashMap::new(), |mut m, op| {
            m.insert(op, args.count);
            m
        });
    let shared_status = Arc::new(RwLock::new(Status::default()));
    let config_file = File::open("config.json").expect("`config.json` not found.");
    info!("Reading config file");
    let cfg: Configuration =
        serde_json::from_reader(config_file).expect("Failed to deserialize configuration.");
    warn!("No status updates are given in the logs. To access that information please use `curl -X GET {} -i", args.address);

    info!("Reading configuration finished");
    debug!("Configuration: {:#?}", cfg);
    let shared_client = Client::new(&cfg)?;
    let client = shared_client.clone();
    let status = Arc::clone(&shared_status);
    info!("Spawning clients");
    thread::Builder::new()
        .name("event_listener".to_owned())
        .spawn(move || {
            let _e = ExitOnPanic;
            debug!("ExitOnPanic installed");
            update_status_according_to_events(client, status);
        })
        .expect("Failed to spawn");
    info!("First client thread spawned");

    let client = shared_client;
    let status = Arc::clone(&shared_status);
    thread::Builder::new()
        .name("transaction_sender".to_owned())
        .spawn(move || {
            let _e = ExitOnPanic;
            debug!("ExitOnPanic installed");
            perform_operations(client, status, args.tps, operations);
        })
        .expect("Failed to spawn");
    info!("Second thread is spawned. Starting server");

    rouille::start_server(args.address, move |_| {
        Response::json(&*shared_status.read().expect("Failed to read json response"))
    });
}

fn update_status_according_to_events(client: Client, status: Arc<RwLock<Status>>) {
    let event_filter = FilterBox::Pipeline(PipelineEventFilter::new());
    for event in client
        .listen_for_events(event_filter)
        .expect("Failed to listen for events")
    {
        if let Ok(Event::Pipeline(event)) = event {
            match event.status {
                PipelineStatus::Validating => {}
                PipelineStatus::Rejected(_) => {
                    status
                        .write()
                        .expect("Failed to lock to write rejection timestamp to status")
                        .tx_is_rejected();
                }
                PipelineStatus::Committed => {
                    status
                        .write()
                        .expect("Failed to lock to write commit timestamp to status")
                        .tx_is_committed();
                }
            }
        } else {
            warn!("TX with unknown status");
            status
                .write()
                .expect("Failed to lock to write unknown status")
                .tx_is_unknown()
        }
    }
}

fn perform_operations(
    client: Client,
    status: Arc<RwLock<Status>>,
    tps: usize,
    mut operations: HashMap<Operation, usize>,
) {
    let interval = if tps == 0 {
        Duration::from_secs_f64(0.)
    } else {
        Duration::from_secs_f64(1_f64 / tps as f64)
    };
    let alice_id = AccountId::from_str("alice@wonderland").expect("Failed to make Alice id");
    let wonderland_id =
        DomainId::new(Name::from_str("wonderland").expect("Failed to create Wodnerland name"));
    while !operations.is_empty() {
        operations.retain(|op, count| {
            let start_time = Instant::now();
            let res = match op {
                Operation::RegisterAccount => {
                    let new_account_name = Name::from_str(format!("alice{}", count).as_str())
                        .expect("Failed to create a new account name");
                    debug!(new_account_name = ?new_account_name, "Submitting a new account");
                    let new_account_id: AccountId =
                        AccountId::new(new_account_name, wonderland_id.clone());
                    let (public_key, _) = KeyPair::generate().unwrap().into();
                    client.submit(RegisterBox::new(Account::new(new_account_id, [public_key])))
                }
                Operation::RegisterDomain => {
                    let new_domain_name = Name::from_str(format!("wonderland{}", count).as_str())
                        .expect("Failed to create a new domain name");
                    debug!(new_domain_name = ?new_domain_name, "Submitting a new domain");
                    let new_domain_id: DomainId = DomainId::new(new_domain_name);
                    client.submit(RegisterBox::new(Domain::new(new_domain_id)))
                }
                Operation::RegisterAssetQuantity => {
                    info!("Submitting rose clone with quantity");
                    let new_asset_name = Name::from_str(format!("rose_quantity{}", count).as_str())
                        .expect("Failed to create a new asset name");
                    debug!(new_asset_name = ?new_asset_name, "Submitting a new asset");
                    let new_asset_definition_id: AssetDefinitionId =
                        AssetDefinitionId::new(new_asset_name, wonderland_id.clone());
                    let new_asset_definition =
                        AssetDefinition::quantity(new_asset_definition_id.clone());
                    let new_asset = Asset::new(
                        AssetId::new(new_asset_definition_id, alice_id.clone()),
                        AssetValue::Quantity(1000),
                    );
                    client.submit_all(vec![
                        RegisterBox::new(new_asset_definition).into(),
                        RegisterBox::new(new_asset).into(),
                    ])
                }
                Operation::RegisterAssetBigQuantity => {
                    info!("Submitting rose clone with big quantity");
                    let new_asset_name =
                        Name::from_str(format!("rose_big_quantity{}", count).as_str())
                            .expect("Failed to create a new asset name");
                    debug!(new_asset_name = ?new_asset_name, "Submitting a new asset");
                    let new_asset_definition_id: AssetDefinitionId =
                        AssetDefinitionId::new(new_asset_name, wonderland_id.clone());
                    let new_asset_definition =
                        AssetDefinition::big_quantity(new_asset_definition_id.clone());
                    let new_asset = Asset::new(
                        AssetId::new(new_asset_definition_id, alice_id.clone()),
                        AssetValue::BigQuantity(100000000999900u128),
                    );
                    client.submit_all(vec![
                        RegisterBox::new(new_asset_definition).into(),
                        RegisterBox::new(new_asset).into(),
                    ])
                }
                Operation::RegisterAssetFixed => {
                    info!("Submitting rose clone with decimal quantity");
                    let new_asset_name = Name::from_str(format!("rose_fixed{}", count).as_str())
                        .expect("Failed to create a new asset name");
                    debug!(new_asset_name = ?new_asset_name, "Submitting a new asset");
                    let new_asset_definition_id: AssetDefinitionId =
                        AssetDefinitionId::new(new_asset_name, wonderland_id.clone());
                    let new_asset_definition =
                        AssetDefinition::fixed(new_asset_definition_id.clone());
                    let new_asset = Asset::new(
                        AssetId::new(new_asset_definition_id, alice_id.clone()),
                        AssetValue::Fixed(Fixed::try_from(1000f64).expect("Valid fixed num")),
                    );
                    client.submit_all(vec![
                        RegisterBox::new(new_asset_definition).into(),
                        RegisterBox::new(new_asset).into(),
                    ])
                }
                Operation::RegisterAssetStore => {
                    info!("Submitting rose clone with metadata");
                    let new_asset_name = Name::from_str(format!("rose_store{}", count).as_str())
                        .expect("Failed to create a new asset name");
                    debug!(new_asset_name = ?new_asset_name, "Submitting a new asset");
                    let new_asset_definition_id: AssetDefinitionId =
                        AssetDefinitionId::new(new_asset_name, wonderland_id.clone());
                    let new_asset_definition =
                        AssetDefinition::store(new_asset_definition_id.clone());
                    let mut store = Metadata::new();
                    store
                        .insert_with_limits(
                            Name::from_str("Bytes").expect("Failed to create a metadata key"),
                            Value::Vec(vec![Value::U32(99), Value::U32(98), Value::U32(300)]),
                            MetadataLimits::new(10, 100),
                        )
                        .expect("Insert some metadata");
                    let new_asset = Asset::new(
                        AssetId::new(new_asset_definition_id, alice_id.clone()),
                        AssetValue::Store(store),
                    );
                    client.submit_all(vec![
                        RegisterBox::new(new_asset_definition).into(),
                        RegisterBox::new(new_asset).into(),
                    ])
                }
            };
            let elapsed = Instant::now().duration_since(start_time);
            if elapsed < interval {
                thread::sleep(interval - elapsed);
            }
            if let Err(err) = res {
                warn!("Submit failed: {}", err);
                true
            } else {
                status
                    .write()
                    .expect("Failed to lock to write status")
                    .tx_is_sent();
                *count -= 1;
                *count != 0
            }
        });
    }
    info!("Submitting empty transactions");
    loop {
        let start_time = Instant::now();
        client
            .submit_all(vec![])
            .expect("Failed to submit empty ISI");
        status
            .write()
            .expect("Failed to lock to write status")
            .tx_is_sent();
        let elapsed = Instant::now().duration_since(start_time);
        if elapsed < interval {
            thread::sleep(interval - elapsed);
        }
    }
}
