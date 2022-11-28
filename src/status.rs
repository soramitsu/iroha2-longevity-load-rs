use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Status {
    txs_committed: usize,
    txs_rejected: usize,
    txs_sent: usize,
    txs_unknown: usize,
    latest_committed_transaction: Option<DateTime<Utc>>,
    latest_rejected_transaction: Option<DateTime<Utc>>,
    latest_sent_at: Option<DateTime<Utc>>,
}

impl Status {
    pub fn tx_is_sent(&mut self, txs_count: usize) -> Option<DateTime<Utc>> {
        self.txs_sent += txs_count;
        self.latest_sent_at.replace(Utc::now())
    }

    pub fn tx_is_committed(&mut self) -> Option<DateTime<Utc>> {
        self.txs_committed += 1;
        self.latest_committed_transaction.replace(Utc::now())
    }

    pub fn tx_is_rejected(&mut self) -> Option<DateTime<Utc>> {
        self.txs_rejected += 1;
        self.latest_rejected_transaction.replace(Utc::now())
    }

    pub fn tx_is_unknown(&mut self) {
        self.txs_unknown += 1;
    }
}
