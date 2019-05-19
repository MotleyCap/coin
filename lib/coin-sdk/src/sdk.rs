use std::collections::{HashMap};
use crate::model::{CoinConfig, Portfolio};
use crate::errors::*;
use crate::account::{Account};
use crate::model::{Asset};
use crate::client_factory::ClientFactory;

pub struct SDK {
  accounts: Vec<Box<Account>>,
}

impl SDK {
  pub fn new(config: CoinConfig) -> Result<Self> {
    let accounts = SDK::get_account_clients(config)?;
    Ok(SDK {
      accounts
    })
  }

  pub fn portfolio(&self) -> Result<Portfolio> {
    Ok(Portfolio {
      balances: vec!(),
      total_usd: 0.0,
      total_btc: 0.0,
    })
  }

  pub fn list_assets(&self, accounts: Option<Vec<String>>) -> Result<Vec<Asset>>  {
    let mut all_balances: HashMap<String, Asset> = HashMap::new();
    for client in &self.accounts {
      let mut add_client_assets = || -> Result<()> {
        let assets = client.list_assets()?;
        for asset in assets {
          if let Some(existing_balance) = all_balances.get(&asset.asset[..]) {
            let new_balance = Asset {
                available: existing_balance.available + asset.available,
                asset: existing_balance.asset.to_owned(),
                locked: existing_balance.locked + asset.locked,
            };
            all_balances.insert(asset.asset.to_owned(), new_balance);
          } else {
            all_balances.insert(asset.asset.to_owned(), asset);
          }
        }
        Ok(())
      };
      if let Some(_accounts) = &accounts {
        if _accounts.contains(&client.name().to_string()) {
          add_client_assets()?;
        }
      } else {
        add_client_assets()?;
      }
    }
    Ok(all_balances
        .iter()
        .map(|(_, val)| val.clone())
        .collect::<Vec<Asset>>())
  }

  fn get_account_clients(config: CoinConfig) -> Result<Vec<Box<Account>>> {
    let mut vec_of_clients: Vec<Box<Account>> = vec!();
    for conf in config.account {
      vec_of_clients.push(ClientFactory::account(conf)?)
    }
    Ok(vec_of_clients)
  }
}