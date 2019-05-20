use std::collections::{HashMap};
use std::fs::File;
use std::io::prelude::*;
use crate::model::{AccountConfig, Asset, Amount};
use crate::errors::*;
use coinbase::account::{Account as AccountImpl};
use coinbase::api::{Coinbase};
use coinbase::model::{CoinbaseTransactionType};
use crate::account::Account;

pub struct CoinbaseAccount {
  config: AccountConfig,
  client: AccountImpl,
}

impl CoinbaseAccount {
  pub fn new(config: AccountConfig) -> Result<Self> {
    match (&config.key, &config.secret) {
      (Some(k), Some(s)) => Ok(CoinbaseAccount {
        client: Coinbase::new(Some(k.to_string()), Some(s.to_string())),
        config
      }),
      _ => bail!("Coinbase account {} requires a key and secret", config.name)
    }
  }
}
impl Account for CoinbaseAccount {

  fn name(&self) -> &str {
    &self.config.name
  }

  fn buy(&self) -> Result<()> {
    Ok(())
  }

  fn sell(&self) -> Result<()> {
    Ok(())
  }

  fn list_assets(&self) -> Result<Vec<Asset>> {
    let coinbase_accounts = self.client.list_accounts()?;
    let mut accounts: Vec<Asset> = vec![];
    // Accumulate by ticker as there can be multiple account types.
    for account in coinbase_accounts.data {
      let account_value: f64 = account.balance.amount.parse()?;
      accounts.push(Asset {
        asset: account.currency.code.to_owned(),
        available: account_value,
        locked: 0.0
      })
    }
    Ok(accounts)
  }

  fn total_costs(&self) -> Result<Amount> {
    let accounts = self.client.list_accounts()?;
    let mut total_cost = 0.0;
    for account in accounts.data {
      let transactions = self.client.list_transactions(&account.id)?;
      let mut file = File::create(format!("coinbase_{}_transactions.json", &account.id))?;
      file.write_all(serde_json::to_string_pretty(&transactions)?.as_bytes())?;
      let acct_cost = transactions.data.iter().filter(
        |transaction| transaction.r#type == CoinbaseTransactionType::Buy
      ).fold(0.0, |acc, x| acc + x.native_amount.amount.parse::<f64>().unwrap_or(0.0));
      total_cost += acct_cost;
    }
    Ok(Amount { amount: total_cost, currency: "USD".to_string() })
  }

  fn total_gains(&self) -> Result<Amount> {
    let accounts = self.client.list_accounts()?;
    let mut total_gains = 0.0;
    for account in accounts.data {
      let transactions = self.client.list_transactions(&account.id)?;
      let acct_cost = transactions.data.iter().filter(
        |transaction| transaction.r#type == CoinbaseTransactionType::Sell
      ).fold(0.0, |acc, x| acc + x.native_amount.amount.parse::<f64>().unwrap_or(0.0));
      total_gains += acct_cost;
    }
    Ok(Amount { amount: total_gains, currency: "USD".to_string() })
  }

  fn cost_basis(&self) -> Result<()> {
    let accounts = self.client.list_accounts()?;
    for account in accounts.data {
      let transactions = self.client.list_transactions(&account.id)?;
      let mut file = File::create(format!("coinbase_{}_transactions.json", &account.id))?;
      file.write_all(serde_json::to_string_pretty(&transactions)?.as_bytes())?;
    }
    Ok(())
  }

  fn capital_gains(&self) -> Result<()> {
    Ok(())
  }
}