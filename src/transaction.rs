use crate::account::Account;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Deposit {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
    amount: f32,
}

#[derive(Deserialize)]
pub struct Withdrawal {
    #[serde(rename(deserialize = "client"))]
    client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    tx_id: u32,
    amount: f32,
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

#[derive(Clone, Copy)]
pub struct DepositedTransaction {
    pub tx_id: u32,
    pub amount: f32,
    pub in_dispute: bool,
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
        eprintln!("processing deposit for account: {}", account.id());

        if !account.is_locked() {
            account.add_deposited_transaction(DepositedTransaction {
                tx_id: self.tx_id,
                amount: self.amount,
                in_dispute: false,
            });
            account.available += self.amount;
            account.total += self.amount;
        } else {
            eprintln!(
                "account {} is locked. ignoring processing transaction.",
                account.id()
            );
        }
    }
}

impl Process for Withdrawal {
    fn process(&self, account: &mut Account) {
        eprintln!("processing withdrawal for account: {}", account.id());

        if !account.is_locked() && account.available >= self.amount {
            account.available -= self.amount;
            account.total -= self.amount;
        } else {
            eprintln!(
                "account {} is locked or has insufficient founds available. ignoring processing transaction.",
                account.id()
            );
        }
    }
}

impl Process for Dispute {
    fn process(&self, account: &mut Account) {
        eprintln!("processing dispute for account: {}", account.id());

        if !account.is_locked() {
            if let Some(transaction) = account.get_deposited_transaction(self.tx_id) {
                if !transaction.in_dispute && account.available >= transaction.amount {
                    account.set_deposited_transaction_as_dispute(self.tx_id);
                    account.available -= transaction.amount;
                    account.held += transaction.amount;
                }
            }
        }
    }
}

impl Process for Resolve {
    fn process(&self, account: &mut Account) {
        eprintln!("processing resolve for account: {}", account.id());

        if !account.is_locked() {
            if let Some(transaction) = account.get_deposited_transaction(self.tx_id) {
                if transaction.in_dispute && account.held >= transaction.amount {
                    account.clear_deposited_transaction_as_dispute(self.tx_id);
                    account.available += transaction.amount;
                    account.held -= transaction.amount;
                }
            }
        }
    }
}

impl Process for Chargeback {
    fn process(&self, account: &mut Account) {
        eprintln!("processing chargeback for account: {}", account.id());

        if !account.is_locked() {
            if let Some(transaction) = account.get_deposited_transaction(self.tx_id) {
                if transaction.in_dispute && account.held >= transaction.amount {
                    account.clear_deposited_transaction_as_dispute(self.tx_id);
                    account.held -= transaction.amount;
                    account.total -= transaction.amount;
                    account.lock();
                }
            }
        }
    }
}
