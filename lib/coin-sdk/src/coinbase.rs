use std::collections::{HashMap};
use crate::model::{AccountConfig, Asset};
use crate::errors::*;
use coinbase::account::{Account as AccountImpl};
use coinbase::api::{Coinbase};
use crate::account::Account;

pub struct CoinbaseAccount {
  config: AccountConfig,
  client: AccountImpl,
}

impl CoinbaseAccount {
  pub fn new(config: AccountConfig) -> Self {
    CoinbaseAccount {
      client: Coinbase::new(Some(config.key.to_string()), Some(config.secret.to_string())),
      config
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

  fn cost_basis(&self) -> Result<()> {
    Ok(())
  }

  fn capital_gains(&self) -> Result<()> {
    Ok(())
  }
}