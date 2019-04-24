use reqwest;
use reqwest::{Method};
use std::time::SystemTime;
use hmac::{Hmac,Mac};
use hex;

static BASE_URI: &'static str = "https://api.coinbase.com";

pub struct Client {
  pub key: String,
  pub secret: String,
}

type CoinbaseResult = reqwest::Result<reqwest::Response>;
impl Client {

    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Client {
            key: api_key.unwrap_or_else(|| "".into()),
            secret: secret_key.unwrap_or_else(|| "".into()),
        }
    }

    pub fn sign(secret: &str, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&secret.as_bytes()).expect("Hmac::new(key)");
        mac.input((timestamp.to_string() + method.as_str() + uri + body_str).as_bytes());
        hex::encode(&mac.result().code())
    }

    fn call_api(&self, method: Method, uri: &str, body_str: &str) -> CoinbaseResult {
        let since_epoch_seconds = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Invalid SystemTime.").as_secs();
        let signature = Self::sign(&self.secret, since_epoch_seconds, method, uri, body_str);
        let url = format!("{}{}", BASE_URI, uri);
        let client = reqwest::Client::new();
        client.get(&url)
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
}
