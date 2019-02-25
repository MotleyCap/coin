use std::collections::{HashMap};
use crate::errors::*;

pub trait ExchangeOps {
  fn name(&self) -> &str;
  fn can_trade(&self) -> bool;
  fn all_balances(&self) -> Result<Vec<Balance>>;
  fn get_balance(&self, symbol: String) -> Result<f64>;
  fn all_prices(&self) -> Result<Vec<Price>>;
  fn get_price(&self, symbol: &str) -> Result<f64>;
  fn market_buy(&self, buy_into: String, buy_with: String, quantity_to_sell: f64) -> Result<Order>;
  fn market_sell(&self, sell_out_of: String, sell_in_to: String, quantity_to_sell: f64) -> Result<Order>;
  fn exit_market(&self, exit_into: String) -> Result<Vec<Order>>;
  fn enter_market(&self, enter_with: String, portfolio: &HashMap<String, f64>) -> Result<Vec<Order>>;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
  pub symbol: String,
  pub id: u64,
  pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub bid_price: f64,
    pub bid_qty: f64,
    pub ask_price: f64,
    pub ask_qty: f64,
}

impl Balance {
  pub fn total(&self) -> f64 {
    self.free + self.locked
  }
}
