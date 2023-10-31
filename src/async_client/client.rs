use std::fmt::Debug;

use super::http::AsyncRequestBuilder;
use color_eyre::eyre::{eyre, Context, Result};
use futures_util::stream::StreamExt;
use hyper::{client::HttpConnector, Client as HyperClient};
use iroha_client::client::{Client as IrohaClient, QueryResponseHandler};
use iroha_crypto::HashOf;
use iroha_data_model::{
    events::pipeline::PipelineRejectionReason, prelude::*, transaction::TransactionPayload,
};
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

#[derive(Debug, Clone)]
pub struct Client {
    iroha_client: IrohaClient,
    hyper_client: HyperClient<HttpConnector>,
}

impl Client {
    #[allow(dead_code)]
    pub async fn submit(
        &self,
        instruction: impl Into<InstructionExpr> + Debug,
    ) -> Result<HashOf<TransactionPayload>> {
        let isi = instruction.into();
        self.submit_all([isi]).await
    }

    pub async fn submit_all(
        &self,
        instructions: impl IntoIterator<Item = InstructionExpr>,
    ) -> Result<HashOf<TransactionPayload>> {
        self.submit_all_with_metadata(instructions, UnlimitedMetadata::new())
            .await
    }

    #[allow(dead_code)]
    pub async fn submit_with_metadata(
        &self,
        instruction: InstructionExpr,
        metadata: UnlimitedMetadata,
    ) -> Result<HashOf<TransactionPayload>> {
        self.submit_all_with_metadata([instruction], metadata).await
    }

    pub async fn submit_all_with_metadata(
        &self,
        instructions: impl IntoIterator<Item = InstructionExpr>,
        metadata: UnlimitedMetadata,
    ) -> Result<HashOf<TransactionPayload>> {
        self.submit_transaction(
            self.iroha_client
                .build_transaction(instructions, metadata)?,
        )
        .await
    }

    pub async fn submit_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> Result<HashOf<TransactionPayload>> {
        self.iroha_client.submit_transaction(&transaction)
    }

    #[allow(dead_code)]
    pub async fn submit_blocking(
        &self,
        instruction: impl Into<InstructionExpr>,
    ) -> Result<SubmitBlockingStatus> {
        self.submit_all_blocking(vec![instruction.into()]).await
    }

    pub async fn submit_all_blocking(
        &self,
        instructions: impl IntoIterator<Item = InstructionExpr>,
    ) -> Result<SubmitBlockingStatus> {
        self.submit_all_blocking_with_metadata(instructions, UnlimitedMetadata::new())
            .await
    }

    pub async fn submit_all_blocking_with_metadata(
        &self,
        instructions: impl IntoIterator<Item = InstructionExpr>,
        metadata: UnlimitedMetadata,
    ) -> Result<SubmitBlockingStatus> {
        let transaction = self
            .iroha_client
            .build_transaction(instructions, metadata)?;
        self.submit_transaction_blocking(transaction).await
    }

    pub async fn submit_transaction_blocking(
        &self,
        transaction: SignedTransaction,
    ) -> Result<SubmitBlockingStatus> {
        let iroha_client = self.iroha_client.clone();
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
        let (init_sender, init_receiver) = oneshot::channel::<()>();
        let hash = transaction.hash();
        spawn(async move {
            let mut event_stream = iroha_client
                .listen_for_events_async(PipelineEventFilter::new().hash(hash.into()).into())
                .await
                .expect("Failed to establish event listener connection.");
            init_sender
                .send(())
                .expect("Failed to send init message through init channel.");
            while let Some(event) = event_stream.next().await {
                let event = event.expect("Failed to listen for the event stream.");
                if let Event::Pipeline(this_event) = event {
                    match this_event.status() {
                        PipelineStatus::Validating => {}
                        PipelineStatus::Rejected(reason) => {
                            return event_sender
                                .send(SubmitBlockingStatus::Rejected(reason.clone()))
                                .expect("Failed to send the transaction through event channel.")
                        }
                        PipelineStatus::Committed => {
                            return event_sender
                                .send(SubmitBlockingStatus::Committed(hash))
                                .expect("Failed to send the transaction through event channel.")
                        }
                    }
                } else {
                    return event_sender
                        .send(SubmitBlockingStatus::Unknown)
                        .expect("Failed to send the transaction through event channel.");
                }
            }
        });
        init_receiver
            .await
            .wrap_err("Failed to receive init message.")?;
        self.submit_transaction(transaction).await?;
        let res = event_receiver
            .recv()
            .await
            .ok_or_else(|| eyre!("Unexpected closing channel"))?;
        Ok(res)
    }
}

#[derive(Debug)]
pub enum SubmitBlockingStatus {
    Committed(HashOf<SignedTransaction>),
    Rejected(PipelineRejectionReason),
    Unknown,
}

impl From<IrohaClient> for Client {
    fn from(iroha_client: IrohaClient) -> Self {
        Self {
            iroha_client,
            hyper_client: HyperClient::new(),
        }
    }
}

impl From<(IrohaClient, HyperClient<HttpConnector>)> for Client {
    fn from((iroha_client, hyper_client): (IrohaClient, HyperClient<HttpConnector>)) -> Self {
        Self {
            iroha_client,
            hyper_client,
        }
    }
}
