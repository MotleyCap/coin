
#[derive(Deserialize, Serialize, Debug)]
pub struct CoinConfig {
    pub blacklist: Option<Vec<String>>,
    pub account: Vec<AccountConfig>,
    pub cmc: CMCConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountConfig {
    pub name: String,
    pub key: String,
    pub secret: String,
    pub passphrase: Option<String>,
    pub readonly: Option<bool>,
    pub service: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CMCConfig {
    pub key: String,
}

/**
 * Exchange specific types.
 */
/**
 * An account contains some value of a single asset and is held
 * by an exchange, by a bank, by a service, or in an offline wallet.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
  pub asset: String,
  pub available: f64,
  pub locked: f64,
}
impl Asset {
  pub fn total(&self) -> f64 {
    self.available + self.locked
  }
}

/**
 * A portfolio is a summary of your entire holdings.
 */

// A single BTC can be divided 100 million times.
// Therefore the smalest unit, called a satoshi, is equal to 0.00000001 BTC
const BTC_FORMAT_MULTIPLIER: f64 = 100000000.0;
const USD_FORMAT_MULTIPLIER: f64 = 100.0;

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
 * A portfolio balance contains information on the holdings of a single currency.
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