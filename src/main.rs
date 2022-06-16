mod account;
mod transaction;

use crate::account::Account;
use crate::transaction::Process;
use crate::transaction::Transaction;

use csv::{ReaderBuilder, Trim};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;

fn get_first_argument() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but none given")),
        Some(file_path) => Ok(file_path),
    }
}

fn process_payments(file_path: OsString) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;

    let mut accounts: HashMap<u16, Account> = HashMap::new();

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(file);
    for result in reader.deserialize() {
        let transaction: Transaction = result?;
        println!("{:?}", transaction);

        let account = accounts
            .entry(transaction.client_id())
            .or_insert_with(|| Account::new(transaction.client_id()));

        transaction.tx_type.process(account);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_argument().expect("file path not provided");

    process_payments(file_path).expect("critical error when processing payments");

    Ok(())
}
