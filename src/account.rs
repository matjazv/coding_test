use crate::transaction::{Deposit, Withdrawal};

use log::{error, info, warn};
use rust_decimal::prelude::*;
use serde::{Serialize, Serializer};

fn to_float<S>(num: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(format!("{:.4}", num).parse().unwrap())
}

#[derive(Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    id: u16,
    #[serde(serialize_with = "to_float")]
    available: Decimal,
    #[serde(serialize_with = "to_float")]
    held: Decimal,
    #[serde(serialize_with = "to_float")]
    total: Decimal,
    locked: bool,
    #[serde(skip_serializing)]
    transactions: Vec<DepositedTransaction>,
}

#[derive(Clone, Copy)]
pub struct DepositedTransaction {
    pub tx_id: u32,
    pub amount: Decimal,
    pub in_dispute: bool,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            id,
            available: Decimal::from(0),
            held: Decimal::from(0),
            total: Decimal::from(0),
            locked: false,
            transactions: Vec::new(),
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn add_transaction(&mut self, transaction: DepositedTransaction) {
        self.transactions.push(transaction);
    }

    pub fn deposit(&mut self, deposit: &Deposit) -> bool {
        if deposit.amount.is_sign_negative() || deposit.amount.is_zero() {
            warn!("tx: {} has zero or negative balance inside", deposit.tx_id);
            return false;
        }

        // In this place an overflow could occurs so it is checked.
        // Discussion is needed if error message is enough of maybe panic should be thrown.
        if self.total.checked_add(deposit.amount).is_none() {
            error!("account {} total amount overflow", self.id);
            return false;
        }
        self.total += deposit.amount;
        self.available += deposit.amount;

        self.add_transaction(DepositedTransaction {
            tx_id: deposit.tx_id,
            amount: deposit.amount,
            in_dispute: false,
        });

        true
    }

    pub fn withdrawal(&mut self, withdrawal: &Withdrawal) -> bool {
        if withdrawal.amount.is_sign_negative() || withdrawal.amount.is_zero() {
            warn!(
                "tx: {} has zero or negative balance inside",
                withdrawal.tx_id
            );
            return false;
        }

        if self.available < withdrawal.amount {
            warn!("account: {} has insufficient funds available", self.id);
            return false;
        }

        self.available -= withdrawal.amount;
        self.total -= withdrawal.amount;

        true
    }

    pub fn set_transaction_as_dispute(&mut self, tx_id: u32) -> bool {
        info!("tx: {} setting as in dispute mode", tx_id);

        for transaction in &mut self.transactions {
            if transaction.tx_id == tx_id
                && !transaction.in_dispute
                && self.available >= transaction.amount
            {
                transaction.in_dispute = true;
                self.available -= transaction.amount;
                self.held += transaction.amount;

                info!("tx: {} successfully set as in dispute mode", tx_id);
                return true;
            }
        }

        warn!("tx: {} is not found, is already in dispute mode or account has insufficient funds available", tx_id);
        false
    }

    pub fn set_transaction_as_resolved(&mut self, tx_id: u32) -> bool {
        info!("tx: {} setting as in resolved mode", tx_id);

        for transaction in &mut self.transactions {
            if transaction.tx_id == tx_id
                && transaction.in_dispute
                && self.held >= transaction.amount
            {
                transaction.in_dispute = false;
                self.available += transaction.amount;
                self.held -= transaction.amount;

                info!("tx: {} successfully set as in resolved mode", tx_id);
                return true;
            }
        }

        warn!("tx: {} is not found, is not in dispute mode or account has insufficient held funds available", tx_id);
        false
    }

    pub fn set_transaction_as_chargeback(&mut self, tx_id: u32) -> bool {
        info!("tx: {} setting as in chargeback mode", tx_id);

        for transaction in &mut self.transactions {
            if transaction.tx_id == tx_id
                && transaction.in_dispute
                && self.held >= transaction.amount
            {
                transaction.in_dispute = false;
                self.held -= transaction.amount;
                self.total -= transaction.amount;
                self.locked = true;

                info!("tx: {} successfully set as in chargeback mode", tx_id);
                return true;
            }
        }

        warn!("tx: {} is not found, is not in dispute mode or account has insufficient held funds available", tx_id);
        false
    }
}
