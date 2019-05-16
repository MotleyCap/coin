use crate::errors::*;
use crate::account::Account;
use crate::model::AccountConfig;
use crate::binance::BinanceAccount;
use crate::coinbase::CoinbaseAccount;
use crate::coinbasepro::CoinbaseProAccount;

pub struct ClientFactory {}

impl ClientFactory {
  pub fn account(config: AccountConfig) -> Result<Box<Account>> {
    match &config.service[..] {
      "binance" => Ok(Box::new(BinanceAccount::new(config))),
      "coinbase" => Ok(Box::new(CoinbaseAccount::new(config))),
      "coinbasepro" => Ok(Box::new(CoinbaseProAccount::new(config)?)),
      serv => Err(ErrorKind::CoinError("Invalid service type ".to_owned() + serv).into())
    }
  }
}