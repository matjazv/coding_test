use crate::account::Account;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Deposit {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
    amount: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct Withdrawal {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
    amount: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(flatten)]
    pub tx_type: TransactionType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    #[serde(rename(deserialize = "deposit"))]
    Deposit(Deposit),
    #[serde(rename(deserialize = "withdrawal"))]
    Withdrawal(Withdrawal),
}

pub trait Process {
    fn process(&self, account: &Account);
}

impl Transaction {
    pub fn client_id(&self) -> u16 {
        match &self.tx_type {
            TransactionType::Deposit(transaction) => transaction.client_id,
            TransactionType::Withdrawal(transaction, ..) => transaction.client_id,
        }
    }
}

impl Process for TransactionType {
    fn process(&self, account: &Account) {
        match self {
            TransactionType::Deposit(transaction) => transaction.process(account),
            TransactionType::Withdrawal(transaction) => transaction.process(account),
        }
    }
}

impl Process for Deposit {
    fn process(&self, account: &Account) {
        println!("Processing deposit for account: {}", account.id());
    }
}

impl Process for Withdrawal {
    fn process(&self, account: &Account) {
        println!("Processing withdrawal for account: {}", account.id());
    }
}
