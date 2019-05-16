use std::collections::{HashMap,HashSet};

#[derive(Deserialize)]
pub struct CMCStatus {
  pub timestamp: Option<String>,
  pub error_code: u64,
  pub error_message: Option<String>,
  pub elapsed: u64,
  pub credit_count: u64,
}
#[derive(Deserialize)]
pub struct CMCQuote {
  pub price: f64,
  pub volume_24h: f64,
  pub percent_change_1h: f64,
  pub percent_change_24h: f64,
  pub percent_change_7d: f64,
  pub market_cap: f64,
  pub last_updated: Option<String>,
}
#[derive(Deserialize)]
pub struct CMCListing {
  pub id: u64,
  pub name: String,
  pub symbol: String,
  pub slug: String,
  pub cmc_rank: u64,
  pub num_market_pairs: u64,
  pub circulating_supply: Option<f64>,
  pub total_supply: Option<f64>,
  pub max_supply: Option<f64>,
  pub last_updated: Option<String>,
  pub date_added: Option<String>,
  pub quote: HashMap<String, CMCQuote>,
}
#[derive(Deserialize)]
pub struct CMCListingResponse {
  pub data: Vec<CMCListing>,
  pub status: CMCStatus,
}
impl CMCListingResponse {
  pub fn fill_usd(mut self) -> Self {
    let mut quote_map = HashMap::new();
    quote_map.insert("USD".to_owned(), CMCQuote {
      price: 1.0,
      volume_24h: 0.0,
      percent_change_1h: 0.0,
      percent_change_24h: 0.0,
      percent_change_7d: 0.0,
      market_cap: 0.0,
      last_updated: None,
    });
    self.data.push(CMCListing {
      id: 99999,
      name: "USD".to_owned(),
      symbol: "USD".to_owned(),
      slug: "USD".to_owned(),
      cmc_rank: 99999,
      num_market_pairs: 1,
      circulating_supply: None,
      total_supply: None,
      max_supply: None,
      last_updated: None,
      date_added: None,
      quote: quote_map,
    });
    self
  }
}
#[derive(Deserialize)]
pub struct CMCHistoricalQuote {
  pub timestamp: String,
  pub quote: HashMap<String, CMCQuote>,
}
#[derive(Deserialize)]
pub struct CMCHistoricalQuotes {
  pub id: u64,
  pub name: String,
  pub symbol: String,
  pub quotes: Vec<CMCHistoricalQuote>,
}
#[derive(Deserialize,Debug)]
pub struct CMCHistoricalQuotesResponse {
  pub result: Vec<(u64, f64)>,
}