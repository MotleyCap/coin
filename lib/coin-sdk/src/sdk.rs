use crate::model::{CoinConfig, Portfolio};
use crate::errors::*;
use crate::account::{Account};
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

  fn get_account_clients(config: CoinConfig) -> Result<Vec<Box<Account>>> {
    let mut vec_of_clients: Vec<Box<Account>> = vec!();
    for conf in config.account {
      vec_of_clients.push(ClientFactory::account(conf)?)
    }
    Ok(vec_of_clients)
  }
}