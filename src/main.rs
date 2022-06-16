mod account;
mod transaction;

use crate::account::Account;
use crate::transaction::Process;
use crate::transaction::Transaction;

use csv::{ReaderBuilder, Trim};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::{env, io};

fn get_file_path() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but none given")),
        Some(file_path) => Ok(file_path),
    }
}

fn save_accounts_data(accounts: &HashMap<u16, Account>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for account in accounts {
        wtr.serialize(account.1)?;
    }
    wtr.flush()?;

    Ok(())
}

fn process_payments(
    file_path: OsString,
    accounts: &mut HashMap<u16, Account>,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(file);
    for result in reader.deserialize() {
        let transaction: Transaction = result?;

        let account = accounts
            .entry(transaction.client_id())
            .or_insert_with(|| Account::new(transaction.client_id()));

        transaction.tx_type.process(account);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let file_path = get_file_path().expect("file path not provided");

    let mut accounts: HashMap<u16, Account> = HashMap::new();
    process_payments(file_path, &mut accounts).expect("critical error when processing payments");

    save_accounts_data(&accounts).expect("can not serialize and save accounts data");

    Ok(())
}
