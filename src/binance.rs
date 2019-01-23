use std::collections::{HashMap};
use binance::api::*;
use binance::account::*;
use binance::market::{Market};
use binance::model::{Prices};
use crate::errors::*;

use crate::model::{ExchangeOps,Balance,Price,Order};

pub struct BinanceClient<'a> {
  pub key: &'a str,
  pub secret: &'a str,
  pub account: Account,
  pub market: Market,
  rounding: HashMap<String, i32>,
}

impl<'a> ExchangeOps for BinanceClient<'a> {
  fn all_balances(&self) -> Result<Vec<Balance>> {
    let balances = match self.account.get_account() {
      Ok(answer) => answer.balances,
      Err(e) => bail!("Error fetching balances: {}", e),
    };
    let coerced = balances
      .iter()
      .map(|bal| Balance { free: bal.free.parse().unwrap(), symbol: bal.asset.to_owned(), locked: bal.locked.parse().unwrap() })
      .collect::<Vec<Balance>>();
    Ok(coerced)
  }

  /**
   * Returns the current holdings for a given symbol as a f64.
   */
  fn get_balance(&self, symbol: String) -> Result<f64> {
    match self.account.get_balance(symbol.to_uppercase()) {
      Err(_) => bail!("Error getting balance for symbol {}", symbol),
      Ok(answer) => answer.free.parse().chain_err(|| format!("Error parsing free balance for symbol {}", symbol))
    }
  }

  /**
   * Return all prices for all tickers.
   */
  fn all_prices(&self) -> Result<Vec<Price>> {
    let prices = match self.market.get_all_prices() {
        Ok(answer) => match answer {
          Prices::AllPrices(ps) => ps
        },
        Err(e) => bail!("Error getting all prices: {}", e),
    };
    let coerced = prices
      .iter()
      .map(|item| Price { symbol: item.symbol.to_owned(), price: item.price })
      .collect::<Vec<Price>>();
    Ok(coerced)
  }

  /**
   * Get a single price by symbol.
   */
  fn get_price(&self, symbol: &str) -> Result<f64> {
    self.market.get_price(symbol).chain_err(|| format!("Could not find symbol {}", symbol))
  }

  /**
   * Buy one currency using some other base currency. You specify how much of the base
   * currency you would like to sell. This method will look up the current spread and
   * determine how much of the new currency to purchase based on the current price.
   */
  fn market_buy(&self, buy_into: String, buy_with: String, quantity_to_sell: f64) -> Result<Order> {
    if quantity_to_sell < 0.001 {
      bail!("Cannot buy {} with {} {}", &buy_into, quantity_to_sell, &buy_with)
    } else {
      let buy_into_u = buy_into.to_uppercase();
      let ticker_name = format!("{}{}",&buy_into_u, buy_with.to_uppercase());
      let ticker_ptr = &ticker_name[..];
      let latest_price = match self.market.get_book_ticker(ticker_ptr) {
        Ok(price) => price.bid_price,
        Err(e) => bail!("Could not fetch book ticker for symbol: {}\n{}", ticker_ptr, e)
      };
      let quantity_to_buy = quantity_to_sell / latest_price;
      let rounding: i32 = match self.rounding.get(&buy_into_u) {
        Some(o) => *o,
        None => 0
      };
      let base_ten: f64 = 10.0;
      let rounding_multiplier = base_ten.powi(rounding);
      let quantity_to_buy_threes = (quantity_to_buy * rounding_multiplier).floor() / rounding_multiplier;
      match self.account.market_buy(ticker_ptr, quantity_to_buy_threes) {
        Ok(answer) => Ok(Order { id: answer.order_id, timestamp: answer.transact_time, symbol: answer.symbol }),
        Err(e) => bail!("Error making market_buy for symbol: {}\n{}", ticker_ptr, e)
      }
    }
  }

  /**
   * Sell on currency into another. You specify how much of the currency you would like to sell
   * and will get be a corresponding amount of the sell_in_to currency.
   */
  fn market_sell(&self, sell_out_of: String, sell_in_to: String, quantity_to_sell: f64) -> Result<Order> {
    let ticker_name = format!("{}{}", sell_out_of.to_uppercase(), sell_in_to.to_uppercase());
    let ticker_ptr = &ticker_name[..];
    let rounding: i32 = match self.rounding.get(&sell_out_of) {
      Some(o) => *o,
      None => 0
    };
    let base_ten: f64 = 10.0;
    let rounding_multiplier = base_ten.powi(rounding);
    let quantity_to_sell_threes = (quantity_to_sell * rounding_multiplier).floor() / rounding_multiplier;
    match self.account.market_sell(ticker_ptr, quantity_to_sell_threes) {
      Ok(answer) => Ok(Order{ id: answer.order_id, symbol: answer.symbol, timestamp: answer.transact_time }),
      Err(e) => bail!("Error making market_sell for symbol: {}\n:{}", ticker_ptr, e)
    }
  }

  /**
   * Sell all owned quantity of a single currency into a base currency.
   */
  fn market_sell_all(&self, sell_out_of: String, sell_in_to: String) -> Result<Order> {
    let sell_out_of_u = sell_out_of.to_uppercase();
    let sell_out_of_u_ptr = &sell_out_of_u[..];
    let balance = match self.account.get_balance(sell_out_of_u_ptr) {
      Ok(answer) => answer.free,
      Err(e) => bail!("Could not get_balance for symbol: {}\n{}", sell_out_of_u_ptr, e)
    };
    let balance_to_sell: f64 = balance.parse().unwrap();
    self.market_sell(sell_out_of, sell_in_to, balance_to_sell)
  }

  /**
   * Exit all holdings into some base currency.
   */
  fn exit_market(&self, exit_into: String) -> Result<Vec<Order>> {
    match self.all_prices() {
      Ok(prices) => {
        let balances = match self.account.get_account() {
          Ok(answer) => answer.balances,
          Err(e) => bail!("Could not get_account: {}", e)
        };
        let mut orders = vec![];
        let exit_into_ptr = &exit_into[..];
        for balance in balances {
          let asset_ptr = &balance.asset[..];
          let total_free = balance.free.parse().unwrap();
          let price_for_asset = match (*prices).iter().find(|p| p.symbol == format!("{}{}", asset_ptr.to_uppercase(), exit_into.to_uppercase())) {
            Some(p) => p.price,
            None => 0.0
          };
          let total_value_in_base = total_free * price_for_asset;
          if asset_ptr != exit_into_ptr && total_value_in_base > 0.001 {
            match self.market_sell(balance.asset, exit_into_ptr.to_owned(), total_free) {
              Ok(sell_order) => orders.push(sell_order),
              Err(e) => println!("Error making market sell {:?}", e)
            }
          }
        }
        Ok(orders)
      },
      Err(e) => Err(Error::with_chain(e, "Error exiting market"))
    }
  }

  /**
   * Enter the market with a portfolio. You provide a base currency which will be used
   * to make all the market_buy orders. The portfolio provides a mapping from asset to
   * percentage of the portfolio that should be dedicated to each currency pair.
   */
  fn enter_market(&self, enter_with: String, portfolio: &HashMap<String, f64>) -> Result<Vec<Order>> {
    let enter_with_ptr = &enter_with[..];
    let base_balance = match self.account.get_balance(enter_with_ptr) {
      Ok(balance) => balance.free,
      Err(e) => bail!("Could not get_balance for symbol: {}\n{}", enter_with_ptr, e)
    };
    let base_balance_f: f64 = base_balance.parse().unwrap();
    let mut orders = vec![];
    for (asset, percentage) in portfolio {
      let asset_ptr = &asset[..];
      if asset_ptr != enter_with_ptr {
        let amount_to_spend = base_balance_f * percentage;
        if let Ok(order_id) = self.market_buy(asset.to_owned(), enter_with_ptr.to_owned(), amount_to_spend) {
          orders.push(order_id);
        }
      }
    }
    Ok(orders)
  }
}

impl<'a> BinanceClient<'a> {
  pub fn new(key: &'a str, secret: &'a str) -> Self {
    let account = Binance::new(Some(key.to_owned()), Some(secret.to_owned()));
    let market: Market = Binance::new(None, None);
    BinanceClient {
      account: account,
      key: key,
      secret: secret,
      market: market,
      rounding: BinanceClient::make_rounding_rules(),
    }
  }

  /**
   * Create a map where the key is the asset name and the value is the number
   * of decimal places that must be rounded to for a minimum trade.
   * https://support.binance.com/hc/en-us/articles/115000594711-Trading-Rule
   */
  fn make_rounding_rules() -> HashMap<String, i32> {
    let mut map = HashMap::new();
    map.insert("ETH".to_owned(), 3);
    map.insert("LTC".to_owned(), 2);
    map.insert("BNB".to_owned(), 1);
    map.insert("NEO".to_owned(), 2);
    map.insert("GAS".to_owned(), 2);
    map.insert("MCO".to_owned(), 2);
    map.insert("WTC".to_owned(), 0);
    map.insert("QTUM".to_owned(), 2);
    map.insert("OMG".to_owned(), 2);
    map.insert("ZRX".to_owned(), 0);
    map.insert("STRAT".to_owned(), 2);
    map.insert("SNGLS".to_owned(), 0);
    map.insert("BQX".to_owned(), 0);
    map.insert("KNC".to_owned(), 0);
    map.insert("FUN".to_owned(), 0);
    map.insert("SNM".to_owned(), 0);
    map.insert("LINK".to_owned(), 0);
    map.insert("XVG".to_owned(), 0);
    map.insert("CTR".to_owned(), 0);
    map.insert("SALT".to_owned(), 2);
    map.insert("IOTA".to_owned(), 0);
    map.insert("MDA".to_owned(), 0);
    map.insert("MTL".to_owned(), 0);
    map.insert("SUB".to_owned(), 0);
    map.insert("EOS".to_owned(), 0);
    map.insert("SNT".to_owned(), 0);
    map.insert("ETC".to_owned(), 2);
    map.insert("MTH".to_owned(), 0);
    map.insert("ENG".to_owned(), 0);
    map.insert("DNT".to_owned(), 0);
    map.insert("BNT".to_owned(), 0);
    map.insert("AST".to_owned(), 0);
    map.insert("DASH".to_owned(), 3);
    map.insert("ICN".to_owned(), 0);
    map.insert("OAX".to_owned(), 0);
    map.insert("BTG".to_owned(), 2);
    map.insert("EVX".to_owned(), 0);
    map.insert("REQ".to_owned(), 0);
    map.insert("LRC".to_owned(), 0);
    map.insert("VIB".to_owned(), 0);
    map.insert("HSR".to_owned(), 0);
    map.insert("TRX".to_owned(), 0);
    map.insert("POWR".to_owned(), 0);
    map.insert("ARK".to_owned(), 2);
    map.insert("YOYO".to_owned(), 0);
    map.insert("XRP".to_owned(), 0);
    map.insert("MOD".to_owned(), 0);
    map.insert("ENJ".to_owned(), 0);
    map.insert("STORJ".to_owned(), 0);
    map.insert("VET".to_owned(), 0);
    map.insert("KMD".to_owned(), 0);
    map.insert("RCN".to_owned(), 0);
    map.insert("NULS".to_owned(), 0);
    map.insert("RDN".to_owned(), 0);
    map.insert("XMR".to_owned(), 3);
    map.insert("DLT".to_owned(), 3);
    map.insert("AMB".to_owned(), 3);
    map.insert("BAT".to_owned(), 0);
    map.insert("ZEC".to_owned(), 3);
    map.insert("BCPT".to_owned(), 0);
    map.insert("ARN".to_owned(), 0);
    map.insert("GVT".to_owned(), 2);
    map.insert("CDT".to_owned(), 0);
    map.insert("GXS".to_owned(), 2);
    map.insert("POE".to_owned(), 0);
    map.insert("QSP".to_owned(), 0);
    map.insert("BTS".to_owned(), 0);
    map.insert("XZC".to_owned(), 2);
    map.insert("LSK".to_owned(), 2);
    map.insert("TNT".to_owned(), 0);
    map.insert("FUEL".to_owned(), 0);
    map.insert("MANA".to_owned(), 0);
    map.insert("BCD".to_owned(), 3);
    map.insert("DGB".to_owned(), 3);
    map.insert("ADX".to_owned(), 0);
    map.insert("ADA".to_owned(), 0);
    map.insert("PPT".to_owned(), 2);
    map.insert("CMT".to_owned(), 0);
    map.insert("XLM".to_owned(), 0);
    map.insert("CND".to_owned(), 0);
    map.insert("LEND".to_owned(), 0);
    map.insert("WABI".to_owned(), 0);
    map.insert("TNB".to_owned(), 0);
    map.insert("WAVES".to_owned(), 2);
    map.insert("ICX".to_owned(), 2);
    map.insert("GTO".to_owned(), 0);
    map.insert("OST".to_owned(), 0);
    map.insert("ELF".to_owned(), 0);
    map.insert("AION".to_owned(), 2);
    map.insert("NEBL".to_owned(), 2);
    map.insert("BRD".to_owned(), 0);
    map.insert("EDO".to_owned(), 2);
    map.insert("WINGS".to_owned(), 0);
    map.insert("NAV".to_owned(), 2);
    map.insert("LUN".to_owned(), 2);
    map.insert("TRIG".to_owned(), 2);
    map.insert("APPC".to_owned(), 0);
    map.insert("VIBE".to_owned(), 0);
    map.insert("RLC".to_owned(), 2);
    map.insert("INS".to_owned(), 2);
    map.insert("PIVX".to_owned(), 2);
    map.insert("IOST".to_owned(), 0);
    map.insert("CHAT".to_owned(), 0);
    map.insert("STEEM".to_owned(), 2);
    map.insert("NANO".to_owned(), 2);
    map.insert("VIA".to_owned(), 2);
    map.insert("BLZ".to_owned(), 0);
    map.insert("AE".to_owned(), 2);
    map.insert("PHX".to_owned(), 0);
    map.insert("NCASH".to_owned(), 0);
    map.insert("POA".to_owned(), 0);
    map.insert("ZIL".to_owned(), 0);
    map.insert("ONT".to_owned(), 0);
    map.insert("STORM".to_owned(), 0);
    map.insert("XEM".to_owned(), 0);
    map.insert("WAN".to_owned(), 0);
    map.insert("QLC".to_owned(), 0);
    map.insert("SYS".to_owned(), 0);
    map.insert("WPR".to_owned(), 0);
    map.insert("GRS".to_owned(), 0);
    map.insert("CLOAK".to_owned(), 2);
    map.insert("GNT".to_owned(), 0);
    map.insert("LOOM".to_owned(), 0);
    map.insert("BCN".to_owned(), 0);
    map.insert("REP".to_owned(), 3);
    map.insert("TUSD".to_owned(), 0);
    map.insert("ZEN".to_owned(), 3);
    map.insert("CVC".to_owned(), 0);
    map.insert("THETA".to_owned(), 0);
    map.insert("IOTX".to_owned(), 0);
    map.insert("QKC".to_owned(), 0);
    map.insert("AGI".to_owned(), 0);
    map.insert("NXS".to_owned(), 2);
    map.insert("DATA".to_owned(), 0);
    map.insert("SC".to_owned(), 0);
    map.insert("NPXS".to_owned(), 1);
    map.insert("KEY".to_owned(), 0);
    map.insert("NAS".to_owned(), 2);
    map.insert("MFT".to_owned(), 0);
    map.insert("DENT".to_owned(), 0);
    map.insert("ARDR".to_owned(), 0);
    map.insert("HOT".to_owned(), 0);
    map.insert("DOCK".to_owned(), 0);
    map.insert("POLY".to_owned(), 0);
    map.insert("GO".to_owned(), 0);
    map.insert("RVN".to_owned(), 0);
    map.insert("DCR".to_owned(), 3);
    map.insert("MITH".to_owned(), 0);
    map.insert("BCHABC".to_owned(), 3);
    map.insert("BCHSV".to_owned(), 3);
    map.insert("REN".to_owned(), 0);
    map
  }
}