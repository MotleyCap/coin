use reqwest::{get};
use std::collections::{HashMap,HashSet};
use std::time::SystemTime;
use crate::model::*;

const CMC_BASE_URL: &str = "https://pro-api.coinmarketcap.com";
const CM_BASE_URL: &str = "https://coinmetrics.io/api";

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