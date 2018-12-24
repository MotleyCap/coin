use binance::api::*;
use binance::account::*;

use crate::clients::client::{ExchangeOps,Balance};

pub struct BinanceClient<'a> {
  pub key: &'a str,
  pub secret: &'a str,
  pub client: Account,
}

impl<'a> ExchangeOps for BinanceClient<'a> {
  fn list(&self) -> Box<Vec<Balance>> {
    let balances = match self.client.get_account() {
      Ok(answer) => answer.balances,
      Err(e) => panic!("Error: {}", e),
    };
    let coerced = balances
      .iter()
      .map(|bal| Balance { free: bal.free.parse().unwrap(), asset: bal.asset.to_owned(), locked: bal.locked.parse().unwrap() })
      .collect::<Vec<Balance>>();
    Box::new(coerced)
  }
}

impl<'a> BinanceClient<'a> {
  pub fn new(key: &'a str, secret: &'a str) -> Self {
    let account = Binance::new(Some(key.to_owned()), Some(secret.to_owned()));
    BinanceClient {
      client: account,
      key: key,
      secret: secret,
    }
  }
}