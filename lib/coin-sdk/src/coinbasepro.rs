use std::collections::{HashMap};
use crate::model::{AccountConfig, Asset, Amount};
use crate::account::{Account};
use crate::errors::*;
use coinbase_pro_rs::{Private, Sync, MAIN_URL};

pub struct CoinbaseProAccount {
  config: AccountConfig,
  private: Private<Sync>,
}

impl CoinbaseProAccount {
  pub fn new(config: AccountConfig) -> Result<CoinbaseProAccount> {
    match (&config.passphrase, &config.key, &config.secret) {
      (Some(ps), Some(k), Some(s)) => Ok(CoinbaseProAccount {
        private: Private::new(MAIN_URL, k, s, ps),
        config,
      }),
      _ => bail!("CoinbasePro accounts requires a key, secret, and passphrase")
    }
  }
}
impl Account for CoinbaseProAccount {

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
    let results = match self.private.get_accounts() {
      Ok(results) => {
        results.into_iter().map(|a| Asset {
          asset: a.currency,
          available: a.available,
          locked: a.hold
        }).collect()
      },
      Err(e) => bail!("Error fetching coinbase pro assets {:?}", e)
    };
    Ok(results)
  }

  fn total_costs(&self) -> Result<Amount> {
    Ok(Amount { amount: 0.0, currency: "USD".to_string() })
  }

  fn total_gains(&self) -> Result<Amount> {
    Ok(Amount { amount: 0.0, currency: "USD".to_string() })
  }

  fn cost_basis(&self) -> Result<()> {
    Ok(())
  }

  fn capital_gains(&self) -> Result<()> {
    Ok(())
  }
}