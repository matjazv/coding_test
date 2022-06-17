# coding_test

This application reads a list of transactions from a .csv file, updates client accounts,
handles disputes and chargebacks, and at the end outputs the state of clients accounts as a CSV to a
standard output.

## Installation
```bash
git clone git@github.com:matjazv/coding_test.git
```

## Running Application
If a list of transactions is in `transactions.csv` and to get final state of clients accounts
into `accounts.csv`, then inside `coding_test` directory execute:
```bash
cargo run -- transactions.csv > accounts.csv
```

## Implementation Notes
* Decimal values: `rust_decimal` crate is used for handling fixed point arithmetic to get a better 
  precision and no rounding errors.
* It is only possible to dispute a deposit type of transactions. Discussion is needed if withdrawals
  also need a dispute option.
* It is not possible to dispute a transaction multiple times. Discussion is needed if this should be
  an option.
* If client does not exist a new entry is added regarding type of transaction. A discussion is needed
  if a new entry is added only if a transaction type is deposit.
