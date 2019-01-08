use reqwest;
use std::collections::{HashMap};
use serde_json::{Value};
use chrono::{DateTime};

const AIRTABLE_BASE_URL: &str = "https://api.airtable.com/v0";

pub struct AirtableClient<'a> {
  api_key: &'a str,
  api_id: &'a str,
  client: reqwest::Client,
}
impl<'a> AirtableClient<'a> {
  pub fn new(key: &'a str, id: &'a str) -> Self {
    let client = reqwest::Client::new();
    AirtableClient {
      api_key: key,
      api_id: id,
      client: client,
    }
  }
  pub fn create_record(&self, base: String, record: &Value) {
    let url = format!("{}/{}/{}", AIRTABLE_BASE_URL, self.api_id, escape_spaces(base));
    println!("Saving to url {}", url);
    let mut data = HashMap::new();
    data.insert("fields", record);
    // TODO: Handle error gracefully.
    let response = match self.client.post(&url)
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&data)
      .send() {
        Ok(mut o) => println!("{:?}", o.text()),
        Err(e) => panic!(e)
      };
  }
}

fn escape_spaces(s: String) -> String {
  s.split(' ').collect::<Vec<&str>>().join("%20")
}