use reqwest::{Client};
use std::collections::{HashMap};
use sha2::{Sha256};
use hmac::{Hmac,Mac};
use coinbase::api::{Coinbase};
use coinbase::account::{Account as CBAccount};
use coinbase::model::*;
use crate::errors::*;
use crate::model::{ExchangeOps,Account,Price,Order,Buy,Amount};

type HmacSha256 = Hmac<Sha256>;

pub struct CoinbaseClient {
  pub name: String,
  pub key: String,
  pub secret: String,
  pub readonly: bool,
  client: Client,
}

type CoinbaseResult = reqwest::Result<reqwest::Response>;
impl CoinbaseClient {

  pub fn list_all_buys(&self) -> Result<Vec<Buy>> {
    let cb: CBAccount = Coinbase::new(Some(self.key.to_string()), Some(self.secret.to_string()));
    let accounts = cb.list_accounts()?;
    let mut all_buys = vec![];
    for acct in accounts.data {
      let buys: CoinbasePaginatedResource<CoinbaseBuy> = cb.list_buys(&acct.id)?;
      for buy in buys.data.iter() {
        all_buys.push(Buy {
          amount: Amount { amount: buy.amount.amount.parse()?, currency: buy.amount.currency.to_string() },
          fee: Amount { amount: buy.fee.amount.parse()?, currency: buy.fee.currency.to_string() },
          subtotal: Amount { amount: buy.subtotal.amount.parse()?, currency: buy.subtotal.currency.to_string() },
          total: Amount { amount: buy.total.amount.parse()?, currency: buy.total.currency.to_string() },
          timestamp: buy.created_at.to_string(),
        });
      }
    }
    Ok(all_buys)
  }
}

impl ExchangeOps for CoinbaseClient {

  fn name(&self) -> &str {
    &self.name[..]
  }

  fn can_trade(&self) -> bool {
    return !self.readonly
  }

  fn all_accounts(&self) -> Result<Vec<Account>> {
    let cb: CBAccount = Coinbase::new(Some(self.key.to_string()), Some(self.secret.to_string()));
    let coinbase_accounts = cb.list_accounts()?;
    let mut accounts: Vec<Account> = vec![];
    // Accumulate by ticker as there can be multiple account types.
    for account in coinbase_accounts.data {
      let account_value: f64 = account.balance.amount.parse()?;
      accounts.push(Account {
        asset: account.currency.code.to_owned(),
        available: account_value,
        locked: 0.0
      })
    }
    Ok(accounts)
  }

  /**
   * Returns the current holdings for a given symbol as a f64.
   */
  fn get_account(&self, asset: String) -> Result<Account> {
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
  fn get_price(&self, asset: &str) -> Result<f64> {
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

impl CoinbaseClient {
  pub fn new(key: String, secret: String, name: String, readonly: bool) -> Self {
    CoinbaseClient {
      name: name,
      key: key,
      readonly: readonly,
      secret: secret,
      client: Client::new()
    }
  }
}
