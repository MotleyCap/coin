use crate::client::{Client};
use crate::account::*;

pub trait Coinbase {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
}

impl Coinbase for Account {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Account {
            client: Client::new(api_key, secret_key),
        }
    }
}