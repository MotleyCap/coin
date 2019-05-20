use crate::client::*;
use crate::model::*;
use crate::errors::*;

pub struct Account {
    pub client: Client
}

impl Account {
    pub fn list_accounts(&self) -> Result<CoinbasePaginatedResource<CoinbaseAccount>> {
        let mut results = self.client.get("/v2/accounts?limit=100")?;
        let coinbase_accounts: CoinbasePaginatedResource<CoinbaseAccount> = results.json()?;
        Ok(coinbase_accounts)
    }

    pub fn list_buys(&self, acct_id: &str) -> Result<CoinbasePaginatedResource<CoinbaseBuy>> {
        let mut resp = self.client.get(&format!("/v2/accounts/{}/buys?limit=100", acct_id))?;
        let mut buys: CoinbasePaginatedResource<CoinbaseBuy> = resp.json()?;
        buys.data.sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(buys)
    }

    pub fn list_transactions(&self, acct_id: &str) -> Result<CoinbasePaginatedResource<CoinbaseTransaction>> {
        let mut resp = self.client.get(&format!("/v2/accounts/{}/transactions?limit=100", acct_id))?;
        let mut transactions: CoinbasePaginatedResource<CoinbaseTransaction> = resp.json()?;
        while transactions.pagination.next_uri.is_some() {
            let url = transactions.pagination.next_uri.as_mut().unwrap().to_string();
            let mut page_resp = self.client.get(&url)?;
            let next_page: CoinbasePaginatedResource<CoinbaseTransaction> = page_resp.json()?;
            transactions.data.extend(next_page.data);
            transactions.pagination = next_page.pagination;
        }
        transactions.data.sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(transactions)
    }
}