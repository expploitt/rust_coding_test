use std::collections::{HashMap, HashSet};
use csv::Terminator;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use crate::error::AppResult;
use std::str::FromStr;

#[derive(Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub type_: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

impl Transaction {
    pub fn new(type_: String, client: u16, tx: u32, amount: Option<f32>) -> Self {
        Transaction { type_, client, tx, amount }
    }
}

#[derive(EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum TransactionOp {
    DEPOSIT,
    WITHDRAWAL,
    DISPUTE,
    RESOLVE,
    CHARGEBACK,
}


#[derive(Serialize, Debug, Clone)]
pub struct Client {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
    #[serde(skip_serializing)]
    dispute_tx: HashSet<u32>,
}

impl Client {
    pub fn new(client: u16) -> Self {
        Client { client, available: 0.0, held: 0.0, total: 0.0, locked: false, dispute_tx: HashSet::<u32>::new() }
    }

    pub fn deposit(&mut self, tx: &Transaction) {
        let deposit = tx.amount.unwrap();
        self.available += deposit;
        self.total += deposit;
    }

    pub fn withdrawal(&mut self, tx: &Transaction) {
        let withdrawal = tx.amount.unwrap();

        if self.available > withdrawal {
            self.available -= withdrawal;
            self.total -= withdrawal;
        }
    }

    pub fn dispute(&mut self, tx: Option<&Transaction>) {
        if let Some(tx) = tx {
            let dispute = tx.amount.unwrap();
            self.available -= dispute;
            self.held += dispute;
            self.dispute_tx.insert(tx.tx);
        }
    }

    pub fn resolve(&mut self, tx: Option<&Transaction>) {
        if let Some(tx) = tx {
            let amount = tx.amount.unwrap();
            self.held -= amount;
            self.available += amount;
            self.dispute_tx.remove(&tx.tx);
        }
    }

    pub fn chargeback(&mut self, tx: Option<&Transaction>) {
        if let Some(tx) = tx {
            let amount = tx.amount.unwrap();
            self.held -= amount;
            self.total -= amount;
            self.locked = true;
        }
    }
}

pub fn process_tx(input_file: &str) -> AppResult<()> {
    let mut clients: HashMap<u16, Client> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();

    let input_file = std::fs::File::open(input_file)?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(input_file);

    let mut record = csv::StringRecord::new();
    let headers = rdr.headers()?.clone();

    while rdr.read_record(&mut record)? {
        let mut tx = record.deserialize::<Transaction>(Some(&headers))?;

        // f32 {.4} format
        if tx.amount.is_some() {
            tx.amount = Some(((tx.amount.unwrap() * 10000.0) as f32).floor() / 10000.0);
        }

        if !clients.contains_key(&tx.client) {
            clients.insert(tx.client, Client::new(tx.client));
        }

        let client = clients.get_mut(&tx.client).unwrap();

        match TransactionOp::from_str(&tx.type_)? {
            TransactionOp::DEPOSIT => {
                client.deposit(&tx);
                transactions.insert(tx.tx.clone(), tx.clone());
            }
            TransactionOp::WITHDRAWAL => {
                client.withdrawal(&tx);
                transactions.insert(tx.tx.clone(), tx.clone());
            }
            TransactionOp::DISPUTE => {
                // Select referenced tx
                client.dispute(transactions.get(&tx.tx));
            }
            TransactionOp::RESOLVE => {
                client.resolve(transactions.get(&tx.tx));
            }
            TransactionOp::CHARGEBACK => {
                client.chargeback(transactions.get(&tx.tx));
            }
        }
    }


    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .terminator(Terminator::CRLF)
        .from_writer(std::io::stdout());


    for (_id, client) in &clients {
        wrt.serialize(client)?;
        wrt.flush()?;
    }

    Ok(())
}