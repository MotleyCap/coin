use std::collections::{HashMap};
use std::time::SystemTime;
use crate::errors::*;
use coinbase_pro_rs::{Private, Sync, MAIN_URL};

use crate::model::{ExchangeOps,Balance,Price,Order};

pub struct CoinbaseProClient {
  pub name: String,
  pub key: String,
  pub secret: String,
  pub passphrase: String,
  pub readonly: bool,
  private: Private<Sync>,
}

impl ExchangeOps for CoinbaseProClient {

  fn name(&self) -> &str {
    &self.name[..]
  }

  fn can_trade(&self) -> bool {
    return !self.readonly
  }

  fn all_balances(&self) -> Result<Vec<Balance>> {
    let results = match self.private.get_accounts() {
      Ok(results) => {
        results.into_iter().map(|a| Balance {
          symbol: a.currency,
          free: a.available,
          locked: a.hold
        }).collect()
      },
      Err(e) => bail!("Error fetching coinbase pro accounts {:?}", e)
    };
    Ok(results)
  }

  /**
   * Returns the current holdings for a given symbol as a f64.
   */
  fn get_balance(&self, symbol: String) -> Result<f64> {
    bail!("Unimplemented")
  }

  /**
   * Return all prices for all tickers.
   */
  fn all_prices(&self) -> Result<Vec<Price>> {
    bail!("Unimplemented")
  }

  /**
   * Get a single price by symbol.
   */
  fn get_price(&self, symbol: &str) -> Result<f64> {
    bail!("Unimplemented")
  }

  /**
   * Buy one currency using some other base currency. You specify how much of the base
   * currency you would like to sell. This method will look up the current spread and
   * determine how much of the new currency to purchase based on the current price.
   */
  fn market_buy(&self, buy_into: String, buy_with: String, quantity_to_sell: f64) -> Result<Order> {
    bail!("Unimplemented")
  }

  /**
   * Sell on currency into another. You specify how much of the currency you would like to sell
   * and will get be a corresponding amount of the sell_in_to currency.
   */
  fn market_sell(&self, sell_out_of: String, sell_in_to: String, quantity_to_sell: f64) -> Result<Order> {
    bail!("Unimplemented")
  }

  /**
   * Exit all holdings into some base currency.
   */
  fn exit_market(&self, exit_into: String) -> Result<Vec<Order>> {
    bail!("Unimplemented")
  }

  /**
   * Enter the market with a portfolio. You provide a base currency which will be used
   * to make all the market_buy orders. The portfolio provides a mapping from asset to
   * percentage of the portfolio that should be dedicated to each currency pair.
   */
  fn enter_market(&self, enter_with: String, portfolio: &HashMap<String, f64>) -> Result<Vec<Order>> {
    bail!("Unimplemented")
  }
}

impl CoinbaseProClient {
  pub fn new(key: String, secret: String, passphrase: String, name: String, readonly: bool) -> Self {
    CoinbaseProClient {
      name: name,
      key: key.to_string(),
      readonly: readonly,
      secret: secret.to_string(),
      passphrase: passphrase.to_string(),
      private: Private::new(MAIN_URL, &key, &secret, &passphrase),
    }
  }
}
