use std::collections::{HashMap};
use crate::model::{AccountConfig};
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

  fn buy(&self) -> Result<()> {
    Ok(())
  }

  fn sell(&self) -> Result<()> {
    Ok(())
  }

  fn list_assets(&self) -> Result<()> {
    Ok(())
  }

  fn cost_basis(&self) -> Result<()> {
    Ok(())
  }

  fn capital_gains(&self) -> Result<()> {
    Ok(())
  }
}