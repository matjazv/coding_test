mod account;
mod transaction;

use crate::account::Account;
use crate::transaction::Process;
use crate::transaction::Transaction;

use csv::{ReaderBuilder, Trim};
use log::error;
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
    let mut writer = csv::Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account.1)?;
    }
    writer.flush()?;

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

    // Here we have an opportunity to make a code to run in parallel.
    // We would need to be careful that for each client only one thread would be in use otherwise
    // it could happen that transactions would not be processed in a correct order.
    // One solution would be that we will have a pool of threads and check if any thread is already
    // processing transaction(s) for a client and if so, send to this thread transaction data
    // (for example, we could use std::sync::mpsc to do that). If there is no thread currently
    // processing client transaction(s) and if any thread is free, use a new thread from a pool
    // to process transaction data for a client.
    for result in reader.deserialize() {
        let transaction: Transaction = match result {
            Ok(transaction) => transaction,
            Err(_) => {
                error!("can not deserialize transaction. skipping it.");
                continue;
            }
        };
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

    // In real world application this data won't be stored in memory (because we could have a lot of data)
    // but in some database or even database + partially in memory to have a quick access.
    let mut accounts: HashMap<u16, Account> = HashMap::new();
    process_payments(file_path, &mut accounts).expect("critical error when processing payments");

    save_accounts_data(&accounts).expect("can not serialize and save accounts data");

    Ok(())
}
