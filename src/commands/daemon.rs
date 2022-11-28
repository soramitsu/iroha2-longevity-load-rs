use super::make_instruction_by_operation;
use crate::{args::RunArgs, number::PositiveFloat, operation::Operation, status::Status};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use hyper::{
    header,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use iroha_client::client::Client;
use iroha_config::client::Configuration;
use iroha_data_model::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::{join, task};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, default_value = "127.0.0.1:8084")]
    address: SocketAddr,
    #[structopt(long, default_value = "2.0")]
    tps: PositiveFloat,
    #[structopt(long, default_value = "100")]
    count: usize,
    #[structopt(long, required = true)]
    operation: Vec<Operation>,
}

#[async_trait]
impl RunArgs for Args {
    async fn run<T: Write + Send>(self, _writer: &mut std::io::BufWriter<T>) -> Result<()> {
        run_daemon(self.address, self.tps, self.count, self.operation).await
    }
}

async fn run_daemon(
    address: SocketAddr,
    tps: PositiveFloat,
    count: usize,
    operations: Vec<Operation>,
) -> Result<()> {
    info!("Welcome to the Iroha 2 longevity load script");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to init logging");
    info!("Staring load script");
    info!("Reading config file");
    let config_file = File::open("config.json").expect("`config.json` not found.");
    let cfg: Configuration =
        serde_json::from_reader(config_file).expect("Failed to deserialize configuration.");
    warn!("No status updates are given in the logs. To access that information please use `curl -X GET {} -i", address);
    info!("Reading configuration finished");
    debug!("Configuration: {:#?}", cfg);
    let operations = operations.into_iter().fold(HashMap::new(), |mut m, op| {
        m.insert(op, count);
        m
    });
    let shared_client = Client::new(&cfg)?;
    let client = shared_client.clone();
    let shared_status = Arc::new(RwLock::new(Status::default()));
    let status = Arc::clone(&shared_status);
    info!("Spawning clients");
    let update_status_fut =
        task::spawn_blocking(move || update_status_according_to_events(client, status));
    info!("First client thread spawned");
    let client = shared_client;
    let status = Arc::clone(&shared_status);
    let perform_operations_fut = task::spawn_blocking(move || {
        let interval = Duration::from_secs_f64(1_f64 / f64::from(tps));
        perform_operations(client.clone(), Arc::clone(&status), interval, operations);
        submit_empty_transactions(client, status, interval);
    });
    info!("Second thread is spawned. Starting server");
    let service = make_service_fn(move |_conn| {
        let status = Arc::clone(&shared_status);

        async move {
            Result::<_, hyper::Error>::Ok(service_fn(move |req| {
                handle_status_request(req, Arc::clone(&status))
            }))
        }
    });
    let server = Server::bind(&address).serve(service);
    join!(
        async {
            update_status_fut
                .await
                .expect("Failed to update status according events");
        },
        async {
            perform_operations_fut
                .await
                .expect("Failed to perform operations");
        },
        async {
            server.await.expect("Failed to serve a service");
        }
    );
    Ok(())
}

fn update_status_according_to_events(client: Client, status: Arc<RwLock<Status>>) {
    let event_filter = FilterBox::Pipeline(PipelineEventFilter::new());
    for event in client
        .listen_for_events(event_filter)
        .expect("Failed to listen for events")
    {
        debug!(event = ?event, "got an event");
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
    interval: Duration,
    mut operations: HashMap<Operation, usize>,
) {
    let alice_id =
        AccountId::from_str("alice@wonderland").expect("Failed to create test account id");
    let wonderland_id =
        DomainId::new(Name::from_str("wonderland").expect("Failed to create test domain name"));
    while !operations.is_empty() {
        operations.retain(|op, count| {
            let start_time = Instant::now();
            debug!(operation = ?op, count = ?count, "perform operation");
            let instructions_iter =
                make_instruction_by_operation(op, alice_id.clone(), wonderland_id.clone(), *count);
            let res: Result<usize> =
                instructions_iter
                    .into_iter()
                    .try_fold(0usize, |acc, instructions| {
                        client.submit_all(instructions)?;
                        Ok(acc + 1)
                    });
            let elapsed = Instant::now().duration_since(start_time);
            if elapsed < interval {
                thread::sleep(interval - elapsed);
            }
            match res {
                Err(err) => {
                    warn!("Submit failed: {}", err);
                    true
                }
                Ok(txs_count) => {
                    status
                        .write()
                        .expect("Failed to lock to write status")
                        .tx_is_sent(txs_count);
                    *count -= 1;
                    *count != 0
                }
            }
        });
    }
}

fn submit_empty_transactions(client: Client, status: Arc<RwLock<Status>>, interval: Duration) {
    info!("Submitting empty transactions");
    loop {
        let start_time = Instant::now();
        client
            .submit_all(vec![])
            .expect("Failed to submit empty ISI");
        status
            .write()
            .expect("Failed to lock to write status")
            .tx_is_sent(1);
        let elapsed = Instant::now().duration_since(start_time);
        if elapsed < interval {
            thread::sleep(interval - elapsed);
        }
    }
}

async fn handle_status_request(
    _req: Request<Body>,
    status: Arc<RwLock<Status>>,
) -> Result<Response<Body>, hyper::Error> {
    let guard = status.read().unwrap();
    let str_status = serde_json::to_string(&*guard).unwrap();
    let res = Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(str_status))
        .unwrap();
    Ok(res)
}
