use std::{collections::{HashMap}, fmt::{self}, io};
use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize,PartialEq)]
pub enum TypeTx 
{
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback
}
impl fmt::Display for TypeTx
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Tx 
{
    pub r#type: TypeTx,
    pub client: u16,
    pub tx: u64,
    pub amount: Option<f64>
}
impl fmt::Display for Tx
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        f.write_str(
            format!("Id: {}, Tx: {}, Type: {}, Amount: {}", 
            self.client, self.tx, self.r#type, self.amount.unwrap_or(0.0)).as_str()
        )   
    }
}

pub struct ClientTransaction
{
    pub amount: f64,
    pub in_dispute: bool,
}

///
/// This represents a clients account and their transaction history
/// 
pub struct Client
{
    /// Account of the client, with the client ID
    pub acc: Account,
    /// History of client transactions (deposits and withdrawals)
    pub history: HashMap<u64,ClientTransaction>,
}
impl Client
{
    ///
    /// Returns a new client with an empty account and history
    /// 
    /// # Arguments
    /// 
    /// * 'name' - The Client ID, as a u64 
    pub fn new(id: u16) -> Client{
        Client { acc: Account::new(id), history:HashMap::new() }
    }
    /// Gets a transaction based on ID, if the client has it
    /// 
    /// # Arguments
    /// 
    /// 'id' - The transaction ID, as u64
    /// 
    /// Realistically this could be a boolean check, but as I use it in
    /// tests later I decided to keep it like this
    pub fn get_transaction(&self, id: &u64) -> Option<&ClientTransaction>
    {
        let out= match self.history.get(id)
        {
            Some(tx) => Some(tx),
            _ => None
        };
        out
    }
    /// Sets a transaction to disputed state, if the client has it
    /// 
    /// # Arguments
    /// 
    /// 'id' - The transaction ID, as u64
    pub fn dispute_transaction(&mut self, id: &u64)
    {
        let try_tx = self.history.get_mut(id);
        match try_tx
        {
            Some(tx) 
            if tx.in_dispute == false => {
                self.acc.held += tx.amount;
                self.acc.available -= tx.amount;
                tx.in_dispute = true;
            },
            _ => ()
        }
    }
    /// Resolves a transaction in a disputed state, if the client has it
    /// 
    /// # Constraint
    /// This can only run if account is not locked
    /// 
    /// # Arguments
    /// 
    /// 'id' - The transaction ID, as u64
    pub fn resolve_transaction(&mut self, id: &u64)
    {
        if self.acc.locked == true{return;}
        let try_tx = self.history.get_mut(id);
        match try_tx
        {
            Some(tx) if tx.in_dispute == true => {
                self.acc.held -= tx.amount;
                self.acc.available += tx.amount;
                tx.in_dispute = false;
            },
            _ => ()
        }
    }
    /// Chargebacks a transaction in a disputed state, if the client has it
    /// This also locks the account
    /// 
    /// # Constraint
    /// This can only run if account is not locked
    /// 
    /// # Arguments
    /// 
    /// 'id' - The transaction ID, as u64
    pub fn chargeback_transaction(&mut self, id: &u64)
    {
        if self.acc.locked == true{return;}
        let try_tx = self.history.get_mut(id);
        match try_tx
        {
            Some(tx) 
            if tx.in_dispute == true => {
                self.acc.held -= tx.amount;
                self.acc.total -= tx.amount;
                self.acc.locked = true;
            },
            _ => ()
        }
    }
    /// Processes a Deposit/Withdrawal style transaction, increasing/decreasing the total/available
    /// and adds it to the history
    /// 
    /// # Constraint
    /// The withdrawal only happens if there are enough funds to support it
    /// This can only run if account is not locked
    /// 
    /// If the account is locked, nothing occurs
    /// 
    /// # Arguments
    /// 
    /// 'tx' - A reference to the transaction
    pub fn process_transaction(&mut self, tx: &Tx)
    {
        if self.acc.locked || self.history.contains_key(&tx.tx) {return}
        let amount = tx.amount.unwrap_or(0f64); //if something went wrong just set it to 0 and move on
        if amount < 0.0 {return}
        match tx.r#type
        {
            TypeTx::Deposit => {
                self.acc.total+=amount;
                self.acc.available+=amount;
                self.history.insert(tx.tx, ClientTransaction{amount, in_dispute:false});
            },
            TypeTx::Withdrawal if self.acc.available > amount => {
                self.acc.total-=amount;
                self.acc.available-=amount;
            },
            _ => ()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account 
{
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool
}
impl Account
{
    pub fn new(id: u16) -> Account{
        Account { client: id, available: 0.0, held: 0.0, total: 0.0, locked: false }
    }
}
impl fmt::Display for Account
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        f.write_str(
            format!(" available: {}, held: {}, total: {}, locked:{}", 
            self.available, self.held, self.total, self.locked).as_str()
        )   
    }
}

/// Writes the resulting accounts to stdout
/// 
/// # Arguments
/// 
/// * 'clients' - The list of clients that have been processed, as a HashMap<u64,Client>
pub fn write_output(clients: HashMap<u16, Client>)
{
    let mut wrtr = csv::Writer::from_writer(io::stdout());
    for c in clients
    {
        if wrtr.serialize(c.1.acc).is_err()
        {
            continue;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn deposit()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.1)};
        client.process_transaction(&tx_deposit);
        assert_eq!(client.acc.total,0.1);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.1);
    }
    #[test]
    fn deposit_lessthan_zero()
    {
        let mut client = Client::new(1);
        let tx_deposit_negative = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(-0.1)};
        client.process_transaction(&tx_deposit_negative);
        assert_eq!(client.acc.total,0.0);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
    }
    #[test]
    fn deposit_history()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.1)};
        let tx_deposit_dupl_id = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(1.0)};
        let tx_deposit_negative = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(-0.1)};
        client.process_transaction(&tx_deposit);
        client.process_transaction(&tx_deposit_negative);
        client.process_transaction(&tx_deposit_dupl_id);
        assert_eq!(client.history.len(),1);
        assert_eq!(client.history.contains_key(&tx_deposit.tx),true);
        assert_ne!(client.history.contains_key(&tx_deposit_negative.tx),false);
        
    }
    #[test]
    fn withdrawal()
    {
        let mut client = Client::new(1);
        client.acc.total = 1.0;
        client.acc.available = 1.0;
        let tx_withdrawal = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_withdrawal);
        assert_eq!(client.acc.total,0.5);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.5);
    }
    #[test]
    fn withdrawal_precision()
    {
        let mut client = Client::new(1);
        client.acc.total = 1.0;
        client.acc.available = 1.0;
        let tx_withdrawal = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:1,amount:Some(0.0001)};
        client.process_transaction(&tx_withdrawal);
        assert_eq!(client.acc.total,0.9999);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.9999);
    }
    #[test]
    fn withdrawal_lessthan_zero()
    {
        let mut client = Client::new(1);
        client.acc.total = 1.0;
        client.acc.available = 1.0;
        let tx_withdrawal = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:1,amount:Some(-0.5)};
        client.process_transaction(&tx_withdrawal);
        assert_eq!(client.acc.total,1.0);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,1.0);
    }
    #[test]
    fn withdrawal_whentotal_zero()
    {
        let mut client = Client::new(1);
        let tx_withdrawal = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_withdrawal);
        assert_eq!(client.acc.total,0.0);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
    }
    #[test]
    fn dispute_transactions()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_deposit.tx);
        let tx_withdrawal = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:2,amount:Some(0.1)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_withdrawal.tx);
        assert_eq!(client.get_transaction(&tx_deposit.tx).unwrap().in_dispute,true);
        assert_eq!(client.get_transaction(&tx_withdrawal.tx).is_none(),true);
        assert_eq!(client.acc.held,0.5);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.5);
    }
    #[test]
    fn dispute_multiple_transactions()
    {
        let mut client = Client::new(1);
        let tx_deposit_a = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        let tx_deposit_b = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:2,amount:Some(0.5)};
        let tx_deposit_c = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:3,amount:Some(0.5)};
        client.process_transaction(&tx_deposit_a);
        client.process_transaction(&tx_deposit_b);
        client.process_transaction(&tx_deposit_c);
        
        client.dispute_transaction(&tx_deposit_b.tx);
        client.dispute_transaction(&tx_deposit_c.tx);

        assert_eq!(client.get_transaction(&tx_deposit_a.tx).unwrap().in_dispute,false);
        assert_eq!(client.get_transaction(&tx_deposit_b.tx).unwrap().in_dispute,true);
        assert_eq!(client.get_transaction(&tx_deposit_c.tx).unwrap().in_dispute,true);
        assert_eq!(client.acc.held,1.0);
        assert_eq!(client.acc.available,0.5);
        assert_eq!(client.acc.total,1.5);
    }
    #[test]
    fn resolve_transactions()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_deposit.tx);
        client.resolve_transaction(&tx_deposit.tx);
        assert_eq!(client.get_transaction(&tx_deposit.tx).unwrap().in_dispute,false);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.5);
        assert_eq!(client.acc.total,0.5);
    }
    #[test]
    fn chargeback_transactions()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        assert_eq!(client.get_transaction(&tx_deposit.tx).unwrap().in_dispute,true);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.0);
    }
    #[test]
    fn chargeback_transaction_twice()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.0);
    }
    #[test]
    fn chargeback_with_disputes()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        let tx_deposit_1 = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:2,amount:Some(1.0)};
        let tx_deposit_2 = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:3,amount:Some(1.0)};
        let tx_deposit_3 = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:4,amount:Some(1.0)};

        client.process_transaction(&tx_deposit);
        client.process_transaction(&tx_deposit_1);
        client.process_transaction(&tx_deposit_2);
        client.process_transaction(&tx_deposit_3);
        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        client.dispute_transaction(&tx_deposit_1.tx);
        client.dispute_transaction(&tx_deposit_2.tx);
        client.dispute_transaction(&tx_deposit_3.tx);

        assert_eq!(client.get_transaction(&tx_deposit_1.tx).unwrap().in_dispute,true);
        assert_eq!(client.get_transaction(&tx_deposit_2.tx).unwrap().in_dispute,true);
        assert_eq!(client.get_transaction(&tx_deposit_3.tx).unwrap().in_dispute,true);
        assert_eq!(client.acc.held,3.0);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,3.0);
    }
    #[test]
    fn missing_transactions()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        client.dispute_transaction(&tx_deposit.tx);
        client.resolve_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        assert_eq!(client.history.contains_key(&tx_deposit.tx),false);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.0);
    }
    #[test]
    fn locked_account()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        let tx_deposit_locked = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:2,amount:Some(0.5)};
        let tx_withdrawal_locked = Tx{r#type:TypeTx::Withdrawal,client:client.acc.client,tx:2,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        client.process_transaction(&tx_deposit_locked);
        client.process_transaction(&tx_withdrawal_locked);
        assert_eq!(client.acc.held,0.0);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.0);
    }
    
    #[test]
    fn locked_account_chargeback()
    {
        let mut client = Client::new(1);
        let tx_deposit = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:1,amount:Some(0.5)};
        let tx_deposit_chargeback = Tx{r#type:TypeTx::Deposit,client:client.acc.client,tx:2,amount:Some(0.5)};
        client.process_transaction(&tx_deposit);
        client.process_transaction(&tx_deposit_chargeback);

        client.dispute_transaction(&tx_deposit.tx);
        client.chargeback_transaction(&tx_deposit.tx);
        
        client.dispute_transaction(&tx_deposit_chargeback.tx);
        client.chargeback_transaction(&tx_deposit_chargeback.tx);
        
        assert_eq!(client.acc.held,0.5);
        assert_eq!(client.acc.available,0.0);
        assert_eq!(client.acc.total,0.5);
    }
}
