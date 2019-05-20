use crate::model::{AccountConfig, Asset, Amount};
use crate::errors::*;
use crate::account::Account;

pub struct OfflineAccount {
  config: AccountConfig,
}

impl OfflineAccount {
  pub fn new(config: AccountConfig) -> Self {
    OfflineAccount {
      config
    }
  }
}
impl Account for OfflineAccount {

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
    match (&self.config.asset, self.config.amount) {
      (Some(a), Some(amt)) => Ok(vec![Asset { asset: a.to_string(), available: amt, locked: 0.0 }]),
      _ => bail!("Offline account {} needs both an 'asset' and 'amount'", self.config.name)
    }
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