use crate::errors::*;
use crate::account::Account;
use crate::model::AccountConfig;
use crate::binance::BinanceAccount;
use crate::coinbase::CoinbaseAccount;
use crate::coinbasepro::CoinbaseProAccount;
use crate::offline::OfflineAccount;

pub struct ClientFactory {}

impl ClientFactory {
  pub fn account(config: AccountConfig) -> Result<Box<Account>> {
    match &config.provider[..] {
      "binance" => Ok(Box::new(BinanceAccount::new(config)?)),
      "coinbase" => Ok(Box::new(CoinbaseAccount::new(config)?)),
      "coinbasepro" => Ok(Box::new(CoinbaseProAccount::new(config)?)),
      "offline" => Ok(Box::new(OfflineAccount::new(config))),
      serv => Err(ErrorKind::CoinError("Invalid service type ".to_owned() + serv).into())
    }
  }
}