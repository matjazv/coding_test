use crate::account::Account;

use log::{info, warn};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Deposit {
    #[serde(rename(deserialize = "client"))]
    pub client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    pub tx_id: u32,
    pub amount: f32,
}

#[derive(Deserialize)]
pub struct Withdrawal {
    #[serde(rename(deserialize = "client"))]
    pub client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    pub tx_id: u32,
    pub amount: f32,
}

#[derive(Deserialize)]
pub struct Dispute {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
}

#[derive(Deserialize)]
pub struct Resolve {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
}

#[derive(Deserialize)]
pub struct Chargeback {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
}

#[derive(Deserialize)]
pub struct Transaction {
    #[serde(flatten)]
    pub tx_type: TransactionType,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    #[serde(rename(deserialize = "deposit"))]
    Deposit(Deposit),
    #[serde(rename(deserialize = "withdrawal"))]
    Withdrawal(Withdrawal),
    #[serde(rename(deserialize = "dispute"))]
    Dispute(Dispute),
    #[serde(rename(deserialize = "resolve"))]
    Resolve(Resolve),
    #[serde(rename(deserialize = "chargeback"))]
    Chargeback(Chargeback),
}

pub trait Process {
    fn process(&self, account: &mut Account);
}

impl Transaction {
    pub fn client_id(&self) -> u16 {
        match &self.tx_type {
            TransactionType::Deposit(transaction) => transaction.client_id,
            TransactionType::Withdrawal(transaction, ..) => transaction.client_id,
            TransactionType::Dispute(transaction, ..) => transaction.client_id,
            TransactionType::Resolve(transaction, ..) => transaction.client_id,
            TransactionType::Chargeback(transaction, ..) => transaction.client_id,
        }
    }
}

impl Process for TransactionType {
    fn process(&self, account: &mut Account) {
        match self {
            TransactionType::Deposit(transaction) => transaction.process(account),
            TransactionType::Withdrawal(transaction) => transaction.process(account),
            TransactionType::Dispute(transaction) => transaction.process(account),
            TransactionType::Resolve(transaction) => transaction.process(account),
            TransactionType::Chargeback(transaction) => transaction.process(account),
        }
    }
}

impl Process for Deposit {
    fn process(&self, account: &mut Account) {
        info!(
            "processing tx: {} (deposit) for account: {}",
            self.tx_id,
            account.id()
        );

        if !account.is_locked() {
            if !account.deposit(self) {
                warn!("can not process deposit for account {}.", account.id());
            }
        } else {
            warn!(
                "account {} is locked. ignoring processing tx.",
                account.id()
            );
        }
    }
}

impl Process for Withdrawal {
    fn process(&self, account: &mut Account) {
        info!(
            "processing tx: {} (withdrawal) for account: {}",
            self.tx_id,
            account.id()
        );

        if !account.is_locked() {
            if !account.withdrawal(self) {
                warn!("can not process withdrawal for account {}.", account.id());
            }
        } else {
            warn!(
                "account {} is locked. ignoring processing tx.",
                account.id()
            );
        }
    }
}

// Currently it's possible only to dispute deposit type of transactions.
// It should be discussed if support for disputing withdrawals is also needed and implement it accordingly.
impl Process for Dispute {
    fn process(&self, account: &mut Account) {
        info!(
            "processing tx: {} (dispute) for account: {}",
            self.tx_id,
            account.id()
        );

        if !account.is_locked() {
            if !account.set_transaction_as_dispute(self.tx_id) {
                warn!(
                    "tx {} can not be set to in dispute mode. ignoring processing tx.",
                    self.tx_id
                );
            }
        } else {
            warn!(
                "account {} is locked. ignoring processing tx.",
                account.id()
            );
        }
    }
}

impl Process for Resolve {
    fn process(&self, account: &mut Account) {
        info!(
            "processing tx: {} (resolve) for account: {}",
            self.tx_id,
            account.id()
        );

        if !account.is_locked() {
            if !account.set_transaction_as_resolved(self.tx_id) {
                warn!(
                    "tx {} can not be set to resolved mode. ignoring processing tx.",
                    self.tx_id
                );
            }
        } else {
            warn!(
                "account {} is locked. ignoring processing tx.",
                account.id()
            );
        }
    }
}

impl Process for Chargeback {
    fn process(&self, account: &mut Account) {
        info!(
            "processing tx: {} (chargeback) for account: {}",
            self.tx_id,
            account.id()
        );

        if !account.is_locked() {
            if !account.set_transaction_as_chargeback(self.tx_id) {
                warn!(
                    "tx {} can not be set to chargeback mode. ignoring processing tx.",
                    self.tx_id
                );
            }
        } else {
            warn!(
                "account {} is locked. ignoring processing tx.",
                account.id()
            );
        }
    }
}
