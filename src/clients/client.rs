use std::collections::{HashMap};

pub trait ExchangeOps {
  fn list(&self) -> Box<Vec<Balance>>;
  fn all_prices(&self) -> Box<Vec<Price>>;
  fn position(&self, symbol: String) -> f64;
  fn market_buy(&self, buy_into: String, buy_with: String, quantity_to_sell: f64) -> u64;
  fn market_sell(&self, sell_out_of: String, sell_in_to: String, quantity_to_sell: f64) -> u64;
  fn market_sell_all(&self, sell_out_of: String, sell_in_to: String) -> u64;
  fn exit_market(&self, exit_into: String) -> Vec<u64>;
  fn enter_market(&self, enter_with: String, portfolio: &HashMap<String, f64>) -> Vec<u64>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Balance {
  pub symbol: String,
  pub free: f64,
  pub locked: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
    pub symbol: String,
    pub price: f64,
}

impl Balance {
  pub fn total(&self) -> f64 {
    self.free + self.locked
  }
}
