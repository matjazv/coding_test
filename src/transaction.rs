use crate::account::Account;

use log::{info, warn};
use rust_decimal::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Deposit {
    #[serde(rename(deserialize = "client"))]
    pub client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    pub tx_id: u32,
    pub amount: Decimal,
}

#[derive(Deserialize)]
pub struct Withdrawal {
    #[serde(rename(deserialize = "client"))]
    pub client_id: u16,
    #[serde(rename(deserialize = "tx"))]
    pub tx_id: u32,
    pub amount: Decimal,
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
            TransactionType::Withdrawal(transaction) => transaction.client_id,
            TransactionType::Dispute(transaction) => transaction.client_id,
            TransactionType::Resolve(transaction) => transaction.client_id,
            TransactionType::Chargeback(transaction) => transaction.client_id,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_client_id_from_transaction() {
        let withdrawal = Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };

        let transaction = Transaction {
            tx_type: TransactionType::Withdrawal(withdrawal),
        };

        assert_eq!(transaction.client_id(), 12345);
    }

    #[test]
    fn test_process_deposit_success() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };

        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };

        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_deposit_account_locked() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };

        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };

        account.locked = true;
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 0);
    }

    #[test]
    fn test_process_withdrawal_success() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let withdrawal = Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Withdrawal(withdrawal),
        };
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_withdrawal_account_locked() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let withdrawal = Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Withdrawal(withdrawal),
        };
        account.locked = true;
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_dispute_success() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_dispute_account_locked() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        account.locked = true;
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_resolve_success() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        transaction.tx_type.process(&mut account);

        let resolve = Resolve {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Resolve(resolve),
        };
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_resolve_account_locked() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        transaction.tx_type.process(&mut account);

        let resolve = Resolve {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Resolve(resolve),
        };
        account.locked = true;
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_chargeback_success() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        transaction.tx_type.process(&mut account);

        let chargeback = Chargeback {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Chargeback(chargeback),
        };
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_process_chargeback_account_locked() {
        let mut account = Account::new(12345);

        let deposit = Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        let transaction = Transaction {
            tx_type: TransactionType::Deposit(deposit),
        };
        transaction.tx_type.process(&mut account);

        let dispute = Dispute {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Dispute(dispute),
        };
        transaction.tx_type.process(&mut account);

        let chargeback = Chargeback {
            client_id: 12345,
            tx_id: 22334456,
        };
        let transaction = Transaction {
            tx_type: TransactionType::Chargeback(chargeback),
        };
        account.locked = true;
        transaction.tx_type.process(&mut account);

        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("125.9999").unwrap());
        assert_eq!(account.total, Decimal::from_str("125.9999").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }
}
