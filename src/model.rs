use crate::errors::*;
use std::collections::HashMap;

// A single BTC can be divided 100 million times.
// Therefore the smalest unit, called a satoshi, is equal to 0.00000001 BTC
const BTC_FORMAT_MULTIPLIER: f64 = 100000000.0;
const USD_FORMAT_MULTIPLIER: f64 = 100.0;

pub trait ExchangeOps {
  fn name(&self) -> &str;
  fn can_trade(&self) -> bool;
  fn all_accounts(&self) -> Result<Vec<Account>>;
  fn get_account(&self, symbol: String) -> Result<Account>;
  fn all_prices(&self) -> Result<Vec<Price>>;
  fn get_price(&self, symbol: &str) -> Result<f64>;
  fn market_buy(&self, buy_into: String, buy_with: String, quantity_to_sell: f64) -> Result<Order>;
  fn market_sell(
    &self,
    sell_out_of: String,
    sell_in_to: String,
    quantity_to_sell: f64,
  ) -> Result<Order>;
  fn exit_market(&self, exit_into: String) -> Result<Vec<Order>>;
  fn enter_market(
    &self,
    enter_with: String,
    portfolio: &HashMap<String, f64>,
  ) -> Result<Vec<Order>>;
}

/**
 * An account contains some value of a single asset and is held
 * by an exchange, by a bank, by a service, or in an offline wallet.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
  pub asset: String,
  pub available: f64,
  pub locked: f64,
}
impl Account {
  pub fn total(&self) -> f64 {
    self.available + self.locked
  }
}

/**
 * A price is the value of some asset relative to some other
 * asset at a particular point in time.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
  pub symbol: String,
  pub price: f64,
}

/**
 * An order is an instance of an order that has been issued to some
 * exchange, bank, or wallet.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
  pub symbol: String,
  pub id: u64,
  pub timestamp: u64,
}

/**
 * A portfolio is a summary of your entire holdings.
 */
#[derive(Deserialize, Serialize, Debug)]
pub struct Portfolio {
  pub balances: Vec<PortfolioBalance>,
  pub total_usd: f64,
  pub total_btc: f64,
}
impl Portfolio {
  pub fn usd(&self) -> f64 {
    (self.total_usd * USD_FORMAT_MULTIPLIER).round() / USD_FORMAT_MULTIPLIER
  }
  pub fn btc(&self) -> f64 {
    (self.total_btc * BTC_FORMAT_MULTIPLIER).round() / BTC_FORMAT_MULTIPLIER
  }
}

/**
 * A portfolio balance contains information for a single balance in the portfolio.
 */
#[derive(Deserialize, Serialize, Debug)]
pub struct PortfolioBalance {
  pub symbol: String,
  pub quantity: f64,
  pub value_usd: f64,
  pub value_btc: f64,
  pub change_7d: f64,
  pub change_24h: f64,
}
impl PortfolioBalance {
  pub fn usd(&self) -> f64 {
    (self.value_usd * USD_FORMAT_MULTIPLIER).round() / USD_FORMAT_MULTIPLIER
  }
  pub fn btc(&self) -> f64 {
    (self.value_btc * BTC_FORMAT_MULTIPLIER).round() / BTC_FORMAT_MULTIPLIER
  }
}
