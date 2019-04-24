use crate::client::*;
use crate::model::*;
use crate::errors::*;

pub struct Account {
    pub client: Client
}

impl Account {
    pub fn list_accounts(&self) -> Result<CoinbasePaginatedResource<CoinbaseAccount>> {
        let mut results = self.client.get("/v2/accounts")?;
        let coinbase_accounts: CoinbasePaginatedResource<CoinbaseAccount> = results.json()?;
        Ok(coinbase_accounts)
    }

    pub fn list_buys(&self, acct_id: &str) -> Result<CoinbasePaginatedResource<CoinbaseBuy>> {
        let mut resp = self.client.get(&format!("/v2/accounts/{}/buys", acct_id))?;
        let buys: CoinbasePaginatedResource<CoinbaseBuy> = resp.json()?;
        Ok(buys)
    }
}