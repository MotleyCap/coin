#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseAccount {
    pub id: String,
    pub name: String,
    pub primary: bool,
    #[serde(rename = "type")]
    pub _type: CoinbaseAccountType,
    pub currency: CoinbaseAccountCurrency,
    pub balance: CoinbaseAmount,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub resource: Option<String>,
    pub resource_path: Option<String>,
    pub allow_deposits: bool,
    pub allow_withdrawals: bool
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
  pub code: String,
  pub name: String,
  pub color: Option<String>,
  pub sort_index: u64,
  pub exponent: u64,
  #[serde(rename = "type")]
  pub _type: Option<String>,
  pub address_regex: Option<String>,
  pub asset_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseAmount {
    pub amount: String,
    pub currency: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseBuy {
  pub id: String,
  pub status: String,
  pub primary: Option<bool>,
  pub payment_method: CoinbaseReference,
  pub transaction: CoinbaseReference,
  pub amount: CoinbaseAmount,
  pub total: CoinbaseAmount,
  pub subtotal: CoinbaseAmount,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
  pub resource: Option<String>,
  pub resource_path: Option<String>,
  pub committed: bool,
  pub instant: bool,
  pub fee: CoinbaseAmount,
  pub payout_at: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbaseReference {
  pub id: String,
  pub resource: String,
  pub resource_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbasePagination {
  pub ending_before: Option<String>,
  pub starting_after: Option<String>,
  pub order: Option<String>,
  pub limit: u64,
  pub previous_uri: Option<String>,
  pub next_uri: Option<String>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CoinbasePaginatedResource<T> {
    pub pagination: CoinbasePagination,
    pub data: Vec<T>
}