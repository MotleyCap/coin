use reqwest;
use std::collections::{HashMap};
use serde_json::{Value, Map};

const AIRTABLE_BASE_URL: &str = "https://api.airtable.com/v0";

#[derive(Deserialize, Serialize, Debug)]
pub struct AirtableConfig {
    pub key: String,
    pub app: String,
    pub table: String,
    pub column_map: Option<ColumnMap>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ColumnMap {
    pub name: Option<String>,
    pub total_btc: Option<String>,
    pub total_usd: Option<String>,
    pub timestamp: Option<String>,
    pub details: Option<String>,
}

pub struct AirtableClient<'a> {
  config: &'a AirtableConfig,
  client: reqwest::Client,
}
impl<'a> AirtableClient<'a> {
  pub fn new(config: &'a AirtableConfig) -> Self {
    let client = reqwest::Client::new();
    AirtableClient {
      config: config,
      client: client,
    }
  }
  fn replace_record(&self, record: Value) -> Value {
    let formatted = if let Some(cm) = &self.config.column_map {
      let mut value = Map::new();
      let btc_key = match &cm.total_btc {
        Some(s) => s.to_string(),
        None => "Value (BTC)".to_string()
      };
      let usd_key = match &cm.total_usd {
        Some(s) => s.to_string(),
        None => "Value (USD)".to_string()
      };
      let details_key = match &cm.details {
        Some(s) => s.to_string(),
        None => "Details".to_string()
      };
      let ts_key = match &cm.timestamp {
        Some(s) => s.to_string(),
        None => "Timestamp".to_string()
      };
      let name_key = match &cm.name {
        Some(s) => s.to_string(),
        None => "Name".to_string()
      };
      value.insert(
        btc_key,
        serde_json::to_value(record.get("total_btc").unwrap().as_f64().unwrap()).unwrap()
      );
      value.insert(
        usd_key,
        serde_json::to_value(record.get("total_usd").unwrap().as_f64().unwrap()).unwrap()
      );
      value.insert(
        details_key,
        serde_json::to_value(record.get("details").unwrap().as_str().unwrap()).unwrap()
      );
      value.insert(
        ts_key,
        serde_json::to_value(record.get("timestamp").unwrap().as_str().unwrap()).unwrap()
      );
      value.insert(
        name_key,
        serde_json::to_value(record.get("name").unwrap().as_str().unwrap()).unwrap()
      );
      serde_json::to_value(value).unwrap()
    } else {
      record
    };
    formatted
  }
  pub fn create_record(&self, record: Value) -> reqwest::Result<String> {
    let url = format!("{}/{}/{}", AIRTABLE_BASE_URL, self.config.app, escape_spaces(self.config.table.to_string()));
    let formatted = self.replace_record(record);
    let mut data = HashMap::new();
    data.insert("fields", formatted);
    // TODO: Handle error gracefully.
    let response = match self.client.post(&url)
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", self.config.key))
      .json(&data)
      .send() {
        Ok(mut d) => d.text(),
        Err(e) => Err(e)
      };
    response
  }
}

fn escape_spaces(s: String) -> String {
  s.split(' ').collect::<Vec<&str>>().join("%20")
}