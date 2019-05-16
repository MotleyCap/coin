use reqwest;
use std::collections::{HashMap};
use serde_json::{Value, Map};
use crate::errors::*;

const AIRTABLE_BASE_URL: &str = "https://api.airtable.com/v0";

#[derive(Deserialize, Serialize, Debug)]
pub struct AirtableConfig {
    pub key: String,
    pub app: String,
    pub table: String,
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

  pub fn create_record(&self, record: Value) -> Result<String> {
    let url = format!("{}/{}/{}", AIRTABLE_BASE_URL, self.config.app, escape_spaces(self.config.table.to_string()));
    let mut data = HashMap::new();
    data.insert("fields", record);
    // TODO: Handle error gracefully.
    let mut response = self.client.post(&url)
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", self.config.key))
      .json(&data)
      .send()
      .chain_err(|| "Unable to put value into airtable.")?;
    let text = response.text().chain_err(|| "Serialize airtable response as text")?;
    Ok(text)
  }
}

fn escape_spaces(s: String) -> String {
  s.split(' ').collect::<Vec<&str>>().join("%20")
}