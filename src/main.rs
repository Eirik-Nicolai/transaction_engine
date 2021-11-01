use std::{collections::HashMap, fs::File};
use csv_transactions::{Client, Tx, TypeTx, write_output};
fn main() 
{
    let input_argument = std::env::args().nth(1);

    if input_argument.is_none()
    {
        //we panic here as we can't really continue without input anyway
        panic!("ERR: No path argument given");        
    }
    let path = input_argument.unwrap();
    let file = match File::open(&path)
    {
        Ok(f) => f,
        Err(_) => {
            //we panic here as we can't really continue without input anyway
            panic!("ERR: Couldn't open file specified");  
        }
    };
    let mut clients = HashMap::new();
    let mut rdr = csv::Reader::from_reader(file);
    for line in rdr.deserialize()
    {  
        let tx: Tx = match line {
            Ok(tx) => tx,
            Err(_)=> {
                continue;
            }
        };
        let c = clients.entry(tx.client).or_insert(Client::new(tx.client));
        let transaction_id = tx.tx;
        match tx.r#type
        {
            TypeTx::Deposit | TypeTx::Withdrawal => {
                c.process_transaction(&tx);
            },
            TypeTx::Dispute => {
                match c.get_transaction(&transaction_id) {
                    Some(_) => {
                        c.dispute_transaction(&transaction_id);
                    },
                    None => ()
                };
            },
            TypeTx::Resolve => {
                match c.get_transaction(&transaction_id) {
                    Some(transaction) => {
                        if transaction.in_dispute
                        {
                            c.resolve_transaction(&transaction_id);
                        }
                            
                    } ,
                    None => ()
                };
            },
            TypeTx::Chargeback => {
                match c.get_transaction(&transaction_id) {
                    Some(transaction) => {
                        if transaction.in_dispute
                        {
                            c.chargeback_transaction(&transaction_id);
                        }
                    },
                    None => ()
                };
            }
        }
    }
    write_output(clients);
}