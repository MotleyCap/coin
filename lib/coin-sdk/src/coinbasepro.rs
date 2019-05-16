use std::collections::{HashMap};
use crate::model::{AccountConfig};
use crate::account::{Account};
use crate::errors::*;
use coinbase_pro_rs::{Private, Sync, MAIN_URL};

pub struct CoinbaseProAccount {
  config: AccountConfig,
  private: Private<Sync>,
}

impl CoinbaseProAccount {
  pub fn new(config: AccountConfig) -> Result<CoinbaseProAccount> {
    if let Some(passphrase) = &config.passphrase {
      Ok(CoinbaseProAccount {
        private: Private::new(MAIN_URL, &config.key.to_string(), &config.secret.to_string(), &passphrase),
        config,
      })
    } else {
      bail!("CoinbasePro accounts require a passphrase")
    }
  }
}
impl Account for CoinbaseProAccount {

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