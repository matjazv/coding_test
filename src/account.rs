use crate::transaction::{Deposit, Withdrawal};

use log::{error, info, warn};
use rust_decimal::prelude::*;
use serde::{Serialize, Serializer};

fn to_float<S>(num: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str = format!("{:.4}", num);
    s.serialize_str(&str)
}

#[derive(Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    id: u16,
    #[serde(serialize_with = "to_float")]
    pub available: Decimal,
    #[serde(serialize_with = "to_float")]
    pub held: Decimal,
    #[serde(serialize_with = "to_float")]
    pub total: Decimal,
    pub locked: bool,
    #[serde(skip_serializing)]
    pub transactions: Vec<DepositedTransaction>,
}

#[derive(Clone, Copy, PartialEq)]
enum DepositedTransactionStatus {
    Accepted,
    Dispute,
    Resolved,
    Chargeback,
}

#[derive(Clone, Copy)]
pub struct DepositedTransaction {
    tx_id: u32,
    amount: Decimal,
    status: DepositedTransactionStatus,
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
            status: DepositedTransactionStatus::Accepted,
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
                && transaction.status == DepositedTransactionStatus::Accepted
                && self.available >= transaction.amount
            {
                transaction.status = DepositedTransactionStatus::Dispute;
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
                && transaction.status == DepositedTransactionStatus::Dispute
                && self.held >= transaction.amount
            {
                // Currently it's not possible to dispute transaction multiple times. If this is
                // a wanted behavior then transaction status should be set to  DepositedTransactionStatus::Accepted
                transaction.status = DepositedTransactionStatus::Resolved;
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
                && transaction.status == DepositedTransactionStatus::Dispute
                && self.held >= transaction.amount
            {
                transaction.status = DepositedTransactionStatus::Chargeback;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::transaction;

    #[test]
    fn test_create_new_account() {
        let account = Account::new(12345);
        assert_eq!(account.id, 12345);
    }

    #[test]
    fn test_get_account_id() {
        let account = Account::new(12345);
        let id = account.id();
        assert_eq!(id, 12345);
    }

    #[test]
    fn test_new_account_is_not_locked() {
        let account = Account::new(12345);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_add_transaction() {
        let mut account = Account::new(12345);
        assert_eq!(account.transactions.len(), 0);

        let transaction = DepositedTransaction {
            tx_id: 123456789,
            amount: Decimal::from_str("12345.6789").unwrap(),
            status: DepositedTransactionStatus::Accepted,
        };
        account.add_transaction(transaction);
        assert_eq!(account.transactions.len(), 1);

        let transaction = account.transactions.get(0).unwrap();
        assert_eq!(transaction.tx_id, 123456789);
        assert_eq!(transaction.amount, Decimal::from_str("12345.6789").unwrap());
        assert!(transaction.status == DepositedTransactionStatus::Accepted);
    }

    #[test]
    fn test_deposit_success() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_deposit_negative_transaction_amount() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("-0.01").unwrap(),
        };
        assert!(!account.deposit(&deposit));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 0);
    }

    #[test]
    fn test_deposit_zero_transaction_amount() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("0").unwrap(),
        };
        assert!(!account.deposit(&deposit));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 0);
    }

    #[test]
    fn test_deposit_overflow_occurs() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::MAX,
        };
        assert!(account.deposit(&deposit));
        assert_eq!(account.transactions.len(), 1);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("1").unwrap(),
        };
        assert!(!account.deposit(&deposit));
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_withdrawal_success() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        let withdrawal = transaction::Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("125.9999").unwrap(),
        };
        assert!(account.withdrawal(&withdrawal));
        assert_eq!(account.available, Decimal::from_str("12219.679").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12219.679").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_withdrawal_negative_transaction_amount() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        let withdrawal = transaction::Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("-100").unwrap(),
        };
        assert!(!account.withdrawal(&withdrawal));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_withdrawal_zero_transaction_amount() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        let withdrawal = transaction::Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("0").unwrap(),
        };
        assert!(!account.withdrawal(&withdrawal));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_withdrawal_insufficient_account_balance() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("100.2222").unwrap(),
        };
        assert!(account.deposit(&deposit));

        let withdrawal = transaction::Withdrawal {
            client_id: 12345,
            tx_id: 22334456,
            amount: Decimal::from_str("100.2223").unwrap(),
        };
        assert!(!account.withdrawal(&withdrawal));
        assert_eq!(account.available, Decimal::from_str("100.2222").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("100.2222").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_dispute_success() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        assert!(account.set_transaction_as_dispute(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_dispute_transaction_does_not_exist() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        assert!(!account.set_transaction_as_dispute(22334456));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_dispute_insufficient_account_balance() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        account.available -= Decimal::from_str("0.0001").unwrap();

        assert!(!account.set_transaction_as_dispute(22334455));
        assert_eq!(account.available, Decimal::from_str("12345.6788").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_dispute_invalid_transaction_status() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));

        let mut transaction = account.transactions.get_mut(0).unwrap();
        transaction.status = DepositedTransactionStatus::Dispute;

        assert!(!account.set_transaction_as_dispute(22334455));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_resolve_success() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        assert!(account.set_transaction_as_resolved(22334455));
        assert_eq!(account.available, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_resolve_transaction_does_not_exist() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        assert!(!account.set_transaction_as_resolved(22334456));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_resolve_insufficient_account_balance() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        account.held -= Decimal::from_str("0.0001").unwrap();

        assert!(!account.set_transaction_as_resolved(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6788").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_resolve_invalid_transaction_status() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        let mut transaction = account.transactions.get_mut(0).unwrap();
        transaction.status = DepositedTransactionStatus::Accepted;

        assert!(!account.set_transaction_as_resolved(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_chargeback_success() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        assert!(account.set_transaction_as_chargeback(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("0").unwrap());
        assert_eq!(account.total, Decimal::from_str("0").unwrap());
        assert!(account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_chargeback_transaction_does_not_exist() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        assert!(!account.set_transaction_as_chargeback(22334456));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_chargeback_insufficient_account_balance() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        account.held -= Decimal::from_str("0.0001").unwrap();

        assert!(!account.set_transaction_as_chargeback(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6788").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }

    #[test]
    fn test_chargeback_invalid_transaction_status() {
        let mut account = Account::new(12345);

        let deposit = transaction::Deposit {
            client_id: 12345,
            tx_id: 22334455,
            amount: Decimal::from_str("12345.6789").unwrap(),
        };
        assert!(account.deposit(&deposit));
        assert!(account.set_transaction_as_dispute(22334455));

        let mut transaction = account.transactions.get_mut(0).unwrap();
        transaction.status = DepositedTransactionStatus::Accepted;

        assert!(!account.set_transaction_as_chargeback(22334455));
        assert_eq!(account.available, Decimal::from_str("0").unwrap());
        assert_eq!(account.held, Decimal::from_str("12345.6789").unwrap());
        assert_eq!(account.total, Decimal::from_str("12345.6789").unwrap());
        assert!(!account.is_locked());
        assert_eq!(account.transactions.len(), 1);
    }
}
