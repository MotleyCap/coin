use reqwest::{Client,Method};
use std::collections::{HashMap};
use sha2::{Sha256};
use hmac::{Hmac,Mac};
use hex;
use std::time::SystemTime;
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
  const BASE_URI: &'static str = "https://api.coinbase.com";
  pub fn sign(secret: &str, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&secret.as_bytes()).expect("Hmac::new(key)");
        mac.input((timestamp.to_string() + method.as_str() + uri + body_str).as_bytes());
        hex::encode(&mac.result().code())
  }

  fn call_api(&self, method: Method, uri: &str, body_str: &str) -> CoinbaseResult {
    let since_epoch_seconds = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Invalid SystemTime.").as_secs();
    let signature = Self::sign(&self.secret, since_epoch_seconds, method, uri, body_str);
    let url = format!("{}{}", Self::BASE_URI, uri);
    self.client.get(&url)
        .header("CB-ACCESS-KEY", self.key.to_owned())
        .header("CB-ACCESS-SIGN", signature)
        .header("CB-ACCESS-TIMESTAMP", since_epoch_seconds.to_string())
        .header("CB-Version", "2019-02-25")
        .send()
  }

  pub fn get(&self, uri: &str) -> CoinbaseResult {
    self.call_api(Method::GET, uri, "")
  }

  pub fn post(&self, uri: &str, body_str: &str) -> CoinbaseResult {
    self.call_api(Method::POST, uri, body_str)
  }

  pub fn list_accounts(&self) -> Result<CoinbasePaginatedResource<CoinbaseAccount>> {
    let results = self.get("/v2/accounts");
    let coinbase_accounts: CoinbasePaginatedResource<CoinbaseAccount> = match results?.json() {
      Ok(o) => o,
      Err(e) => bail!(Error::with_chain(e, "Error parsing coinbase response"))
    };
    Ok(coinbase_accounts)
  }

  pub fn list_buys(&self) -> Result<Vec<Buy>> {
    let accounts = self.list_accounts()?;
    let mut all_buys = vec![];
    for acct in accounts.data {
      let mut resp = self.get(&format!("/v2/accounts/{}/buys", acct.id))?;
      let buys: CoinbasePaginatedResource<CoinbaseBuy> = resp.json()?;
      for buy in buys.data.iter() {
        let created = if let Some(d) = &buy.created_at { d.to_string() } else { "Unknown".to_string() };
        all_buys.push(Buy {
          amount: Amount { amount: buy.amount.amount.parse()?, currency: buy.amount.currency.to_string() },
          fee: Amount { amount: buy.fee.amount.parse()?, currency: buy.fee.currency.to_string() },
          subtotal: Amount { amount: buy.subtotal.amount.parse()?, currency: buy.subtotal.currency.to_string() },
          total: Amount { amount: buy.total.amount.parse()?, currency: buy.total.currency.to_string() },
          timestamp: created,
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
    let coinbase_accounts = self.list_accounts()?;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseAccount {
    id: String,
    name: String,
    primary: bool,
    #[serde(rename = "type")]
    _type: CoinbaseAccountType,
    currency: CoinbaseAccountCurrency,
    balance: CoinbaseAmount,
    created_at: Option<String>,
    updated_at: Option<String>,
    resource: Option<String>,
    resource_path: Option<String>,
    allow_deposits: bool,
    allow_withdrawals: bool
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum CoinbaseAccountType {
  #[serde(rename = "wallet")]
  Wallet,
  #[serde(rename = "fiat")]
  Fiat,
  #[serde(rename = "vault")]
  Vault
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseAccountCurrency {
  code: String,
  name: String,
  color: Option<String>,
  sort_index: u64,
  exponent: u64,
  #[serde(rename = "type")]
  _type: Option<String>,
  address_regex: Option<String>,
  asset_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseAmount {
    amount: String,
    currency: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseBuy {
  id: String,
  status: String,
  primary: Option<bool>,
  payment_method: CoinbaseReference,
  transaction: CoinbaseReference,
  amount: CoinbaseAmount,
  total: CoinbaseAmount,
  subtotal: CoinbaseAmount,
  created_at: Option<String>,
  updated_at: Option<String>,
  resource: Option<String>,
  resource_path: Option<String>,
  committed: bool,
  instant: bool,
  fee: CoinbaseAmount,
  payout_at: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseReference {
  id: String,
  resource: String,
  resource_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbasePagination {
  ending_before: Option<String>,
  starting_after: Option<String>,
  order: Option<String>,
  limit: u64,
  previous_uri: Option<String>,
  next_uri: Option<String>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbasePaginatedResource<T> {
  pagination: CoinbasePagination,
  data: Vec<T>
}