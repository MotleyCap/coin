use reqwest::{get};
use std::collections::{HashMap,HashSet};
use std::time::SystemTime;

const CMC_BASE_URL: &str = "https://pro-api.coinmarketcap.com";
const CM_BASE_URL: &str = "https://coinmetrics.io/api";

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
pub struct CMCClient {
  pub key: String,
}
impl CMCClient {
  pub fn new(key: String) -> Self {
    CMCClient {
      key: key
    }
  }
  pub fn latest_listings(&self, limit: u16) -> CMCListingResponse {
    let url: &str = &format!(
      "{}/v1/cryptocurrency/listings/latest?limit={}&CMC_PRO_API_KEY={}",
      CMC_BASE_URL,
      limit,
      self.key
    );
    let body: CMCListingResponse = match get(url) {
      Ok(mut data) => match data.json() {
        Ok(o) => o,
        Err(e) => {
          println!("{:?}", e);
          panic!(e);
        }
      },
      Err(_) => CMCListingResponse {
        data: vec![],
        status: CMCStatus {
          timestamp: Option::None,
          error_code: 500,
          error_message: Option::None,
          elapsed: 1,
          credit_count: 0,
        }
      },
    };
    body.fill_usd()
  }

  pub fn historic_quotes(&self, symbol: &str, count: u64, _interval: &str) -> CMCHistoricalQuotesResponse {
    let current_epoch = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(), // println!("1970-01-01 00:00:00 UTC was {} seconds ago!", ),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let seconds_in_day = 60 * 60 * 24;
    let extra_seconds = current_epoch % seconds_in_day;
    let beginning_of_today = current_epoch - extra_seconds;
    let beginning_of_period = beginning_of_today - (seconds_in_day * count);
    let l_symbol = &symbol.to_lowercase()[..];
    let url: &str = &format!(
      "{}/v1/get_asset_data_for_time_range/{}/marketcap(usd)/{}/{}",
      CM_BASE_URL,
      l_symbol,
      beginning_of_period,
      beginning_of_today
    );
    let body: CMCHistoricalQuotesResponse = match get(url) {
      Ok(mut data) => {
        match data.json() {
          Ok(o) => o,
          Err(e) => {
            println!("{}",e);
            println!("{:?}", data);
            panic!(e);
          },
        }
      },
      Err(e) => panic!(e),
    };
    body
  }

  pub fn supported_assets(&self) -> HashSet<String> {
    let url: &str = &format!(
      "{}/v1/get_supported_assets",
      CM_BASE_URL
    );
    let body: HashSet<String> = match get(url) {
      Ok(mut data) => match data.json() {
        Ok(o) => o,
        Err(e) => {
          println!("{}",e);
          println!("{:?}", data);
          panic!(e);
        },
      },
      Err(e) => panic!(e),
    };
    body.iter().map(|s| s.to_uppercase()).collect()
  }
}