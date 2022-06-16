use crate::transaction::DepositedTransaction;

use log::info;
use serde::{Serialize, Serializer};

fn float_precision_serialize<S>(num: &f32, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f32(format!("{:.4}", num).parse().unwrap())
}

#[derive(Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    id: u16,
    #[serde(serialize_with = "float_precision_serialize")]
    pub available: f32,
    #[serde(serialize_with = "float_precision_serialize")]
    pub held: f32,
    #[serde(serialize_with = "float_precision_serialize")]
    pub total: f32,
    locked: bool,
    #[serde(skip_serializing)]
    deposited_transactions: Vec<DepositedTransaction>,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
            deposited_transactions: Vec::new(),
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        info!("locking account {}", self.id);
        self.locked = true;
    }

    pub fn add_deposited_transaction(&mut self, transaction: DepositedTransaction) {
        self.deposited_transactions.push(transaction);
    }

    pub fn get_deposited_transaction(&self, tx_id: u32) -> Option<DepositedTransaction> {
        for transaction in &self.deposited_transactions {
            if transaction.tx_id == tx_id {
                return Some(*transaction);
            }
        }

        None
    }

    pub fn set_deposited_transaction_as_dispute(&mut self, tx_id: u32) {
        for mut transaction in &mut self.deposited_transactions {
            if transaction.tx_id == tx_id {
                info!("setting tx: {} in dispute mode", tx_id);
                transaction.in_dispute = true;
            }
        }
    }

    pub fn clear_deposited_transaction_as_dispute(&mut self, tx_id: u32) {
        for mut transaction in &mut self.deposited_transactions {
            if transaction.tx_id == tx_id {
                info!("clearing tx: {} in dispute mode", tx_id);
                transaction.in_dispute = false;
            }
        }
    }
}
