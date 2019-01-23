// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

use std::fs;
use std::collections::{HashMap,HashSet};
use dirs::{home_dir};

use clap::clap_app;
use chrono::prelude::*;

mod portfolio;
mod cmc;
mod model;
mod binance;
mod airtable;

use crate::binance::{BinanceClient};
use crate::model::{ExchangeOps,Balance,Price};
use crate::cmc::{CMCClient, CMCListingResponse, CMCListing};
use crate::airtable::{AirtableClient,AirtableConfig};
use crate::portfolio::{Portfolio};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;

use colored::*;
use prettytable::{Table, row, cell};

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

// A single BTC can be divided 100 million times.
// Therefore the smalest unit, called a satoshi, is equal to 0.00000001 BTC
const BTC_FORMAT_MULTIPLIER: f64 = 100000000.0;
const USD_FORMAT_MULTIPLIER: f64 = 100.0;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
use crate::errors::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

// The above main gives you maximum control over how the error is
// formatted. If you don't care (i.e. you want to display the full
// error during an assert) you can just call the `display_chain` method
// on the error object
#[allow(dead_code)]
fn alternative_main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

// Use this macro to auto-generate the main above. You may want to
// set the `RUST_BACKTRACE` env variable to see a backtrace.
// quick_main!(run);

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<()> {
    let matches = matches();
    let coin_file = if let Some(p) = home_dir() {
        match fs::read_to_string(p.join(".coin.toml")) {
            Ok(contents) => {
                Some(contents)
            },
            _ => bail!("Error reading ~/.coin.toml")
        }
    } else {
        bail!("Could not find ~/.coin.toml")
    };
    let raw_config: Option<Config> = match &coin_file {
        Some(contents) => {
            let conf: Config = match toml::from_str(&contents) {
                Ok(conts) => conts,
                Err(e) => bail!(Error::with_chain(e, "Error parsing .coin.toml"))
            };
            Some(conf)
        },
        None => None
    };
    let config: Config = match raw_config {
        None => bail!("Could not find ~/.coin.toml"),
        Some(c) => c
    };
    let key = config.binance.key.to_owned();
    let secret = config.binance.secret.to_owned();
    let cmc_key = config.cmc.key.to_owned();
    let blacklisted_symbols: HashSet<_> = match &config.blacklist {
        Some(l) => {
            let as_set: HashSet<String> = l.iter().map(|s| s.to_string()).collect();
            as_set
        },
        None => HashSet::new()
    };
    let airtable_config = config.airtable.unwrap_or(AirtableConfig { key: "".to_string(), app: "".to_string(), table: "".to_string(), column_map: None});
    let airtable = if (&airtable_config.key).len() > 0 && (&airtable_config.app).len() > 0 {
        Some(AirtableClient::new(&airtable_config))
    } else {
        None
    };
    let binance = BinanceClient::new(&key[..], &secret[..]);
    let cmc = CMCClient::new(cmc_key);
    if let Some(_matches) = matches.subcommand_matches("account") {
        match binance.all_balances() {
            Ok(balances) => {
                let prices = cmc.latest_listings(100);
                match make_account(&balances, prices) {
                    Ok(acct) => {
                        print_account(&acct);
                        return Ok(())
                    },
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    } else if let Some(_matches) = matches.subcommand_matches("save") {
        match binance.all_balances() {
            Ok(balances) => {
                let prices = cmc.latest_listings(100);
                match make_account(&balances, prices) {
                    Ok(account) => {
                        if let Some(a_t) = airtable {
                            save_account(&a_t, &account);
                        } else {
                            println!("Could not find airtable credentials in ~/.coin.yaml");
                        }
                        Ok(())
                    },
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    } else if let Some(_matches) = matches.subcommand_matches("symbols") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC").to_ascii_uppercase();
        let tradable_symbols = get_tradeable_symbols(&base_currency, &blacklisted_symbols, &binance, &cmc);
        println!("Tradable symbols: {:?}", tradable_symbols);
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("cmc") {
        let prices = cmc.latest_listings(100);
        print_cmc_listings(&prices);
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("config") {
        println!("{}", coin_file.unwrap_or("Could not find ~/.coin.env".to_owned()));
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("binance") {
        match binance.all_prices() {
            Ok(prices) => {
                print_prices(prices);
                Ok(())
            },
            Err(e) => Err(e)
        }
    } else if let Some(_matches) = matches.subcommand_matches("balance") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC").to_uppercase();
        if base_currency != "BTC" {
            bail!("BTC is currently the only supported base currency for balance operations.")
        }
        let index_size = _matches.value_of("size").unwrap_or("10");
        let index_size_i: u64 = index_size.parse().unwrap();
        let lookback = _matches.value_of("lookback").unwrap_or("20");
        let lookback_i: u64 = lookback.parse().unwrap();
        let factor = _matches.value_of("factor").unwrap_or("0.3");
        let factor_i: f64 = factor.parse().unwrap();
        let is_mock = _matches.is_present("mock");
        // Find all pairs that trade with the base pai
        let tradable_symbols = get_tradeable_symbols(&base_currency, &blacklisted_symbols, &binance, &cmc)?;
        // First exit the market to the base currency.
        if !is_mock {
            if let Ok(orders) = binance.exit_market(base_currency.to_owned()) {
                let order_ids_str = orders.iter().map(|o| o.id.to_string()).collect::<Vec<String>>();
                println!("Successfully exited old positions with order_ids: [{}]", order_ids_str.join(", ").blue());
            }
        }
        let cmc_prices = cmc.latest_listings(100);
        let balanced_portfolio = balance_by_market_cap(&cmc, &cmc_prices.data, index_size_i, lookback_i, factor_i, tradable_symbols);
        print_portfolio(&balanced_portfolio);
        // Calculating total value
        if !is_mock {
            match binance.enter_market(base_currency.to_owned(), &balanced_portfolio) {
                Ok(vec_of_orders) => {
                    let order_ids_str = vec_of_orders.iter().map(|o| o.id.to_string()).collect::<Vec<String>>();
                    if let Some(a_t) = airtable {
                        if let Ok(balances) = binance.all_balances() {
                            // let account = make_account(&balances, cmc_prices);
                            if let Ok(account) = make_account(&balances, cmc_prices) {
                                save_account(&a_t, &account)
                            }
                        } else {
                            println!("Error saving to airtable");
                        }
                    }
                    println!("Successfully entered new positions with order_ids: [{}]", order_ids_str.join(", ").blue());
                    Ok(())
                },
                Err(e) => Err(Error::with_chain(e, "Error entering market."))
            }
        } else {
            Ok(())
        }
    } else if let Some(_matches) = matches.subcommand_matches("buy") {
        let amount_to_buy: f64 = match _matches.value_of("amount") {
            Some(a) => {
                if let Ok(_a) = a.parse::<f64>() {
                    _a
                } else {
                    bail!("Invalid amount {}", a)
                }
            },
            None => bail!("You must provide an amount to buy.")
        };
        let asset_to_buy = match _matches.value_of("asset") {
            Some(at) => at.to_uppercase(),
            None => bail!("You must provide the symbol of the asset you want to buy.")
        };
        let asset_to_buy_with = _matches.value_of("with").unwrap_or("BTC").to_uppercase();
        match binance.market_buy(asset_to_buy, asset_to_buy_with, amount_to_buy) {
            Ok(order) => {
                println!("Successfully bought {:?}", order);
                Ok(())
            },
            Err(e) => Err(e)
        }
    } else if let Some(_matches) = matches.subcommand_matches("exit") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC");
        if _matches.is_present("get_balance") {
            let position_to_exit = _matches.value_of("get_balance").unwrap();
            match binance.market_sell_all(position_to_exit.to_owned(), base_currency.to_owned()) {
                Ok(order) => {
                    println!("Successfully exited {} with order_id: {}", position_to_exit, order.id.to_string().green());
                    Ok(())
                },
                Err(e) => Err(e)
            }
        } else {
            match binance.exit_market(base_currency.to_owned()) {
                Ok(orders) => {
                    let order_ids_str = orders.iter().map(|o| o.id.to_string()).collect::<Vec<String>>();
                    println!("Successfully exited all positions with order_ids: [{}]", order_ids_str.join(", ").blue());
                    Ok(())
                },
                Err(e) => Err(e)
            }
        }
    } else {
        bail!("Unknown command")
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    pub blacklist: Option<Vec<String>>,
    pub binance: BinanceConfig,
    pub cmc: CMCConfig,
    pub airtable: Option<AirtableConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
struct BinanceConfig {
    pub key: String,
    pub secret: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CMCConfig {
    pub key: String,
}

fn get_tradeable_symbols(base_currency: &str, blacklist: &HashSet<String>, binance: &BinanceClient, cmc: &CMCClient) -> Result<HashSet<String>> {
    match binance.all_prices() {
        Ok(prices) => {
            let mut tradable_symbols: HashSet<_> = prices
                .iter()
                .filter(
                    |item| item.symbol.ends_with(&base_currency) || item.symbol.starts_with(&base_currency)
                ).map(
                    |item| match item.symbol.ends_with(&base_currency) {
                        true => item.symbol[0..item.symbol.len()-base_currency.len()].to_owned(),
                        false => item.symbol[base_currency.len()..item.symbol.len()].to_owned()
                    }
                ).collect();
            let coins_with_data = cmc.supported_assets();
            println!("Fetched {} supported assets from api.coinmetrics.com", coins_with_data.len());
            if !tradable_symbols.contains(base_currency) {
                tradable_symbols.insert(base_currency.to_string());
            }
            let tradable_symbols: HashSet<String> = tradable_symbols.difference(blacklist).map(|s| s.to_string()).collect();
            let tradable_symbols: HashSet<String> = tradable_symbols.intersection(&coins_with_data).map(|s| s.to_string()).collect();
            Ok(tradable_symbols)
        },
        Err(e) => Err(e)
    }
}

#[derive(Deserialize,Serialize)]
struct AccountRecord {
    total_usd: f64,
    total_btc: f64,
    timestamp: String,
    details: String
}
fn save_account(airtable: &AirtableClient, account: &Account) {
    let now = Utc::now();
    let account_str = serde_json::to_string_pretty(&account.balances).unwrap();
    let value = AccountRecord {
        total_usd: account.total_usd,
        total_btc: account.total_btc,
        timestamp: now.to_string(),
        details: account_str
    };
    let value_json = serde_json::to_value(value).unwrap();
    match airtable.create_record(value_json) {
        Ok(_) => println!("Successfully saved portfolio to airtable."),
        Err(e) => println!("Error saving portfolio to airtable: ${}", e.status().unwrap())
    };
}

fn balance_by_market_cap(
    cmc: &CMCClient,
    prices: &Vec<CMCListing>,
    index_size: u64,
    lookback: u64,
    smoothing_factor: f64,
    tradable_assets: HashSet<String>
) -> HashMap<String, f64> {
    let mut market_caps = HashMap::new();
    let mut seen_assets = 0;
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Count", "Historical Market Caps"]);
    for price in prices {
        if seen_assets >= index_size {
            break;
        }
        let l_symbol = price.symbol.to_uppercase()[..].to_owned();
        if !tradable_assets.contains(&l_symbol) {
            continue;
        }
        let historical_quotes = cmc.historic_quotes(&price.symbol, lookback, "daily");
        // Slow rate to 5 reqs a second
        // let throttle_length = time::Duration::from_millis(200);
        // thread::sleep(throttle_length);
        let historical_market_caps = historical_quotes.result.iter().map(
            |h_quote| {
                let price = h_quote.1;
                price
            }
        ).collect::<Vec<f64>>();
        let symbol = &price.symbol[..];
        let values_as_string = historical_market_caps.iter().map(|f| f.to_string()).collect::<Vec<String>>().join(",");
        let len = historical_market_caps.len();
        market_caps.insert((&price.symbol).to_owned(), historical_market_caps);
        table.add_row(row![
            symbol,
            len,
            values_as_string,
        ]);
        seen_assets = seen_assets + 1;
    }
    let portfolio = Portfolio::new(market_caps, smoothing_factor);
    let allotments = portfolio.balance_by_market_cap();
    // table.printstd();
    allotments
}

fn print_prices(prices: Vec<Price>) {
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Price"]);
    for price in prices.iter() {
        table.add_row(row![price.symbol, format!("{:.5}", price.price)]);
    }
    table.printstd();
}

fn print_portfolio(allotments: &HashMap<String, f64>) {
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Percentage"]);
    for (symbol, percentage) in allotments {
        table.add_row(row![symbol, format!("{:.2}", percentage * 100.0)]);
    }
    table.printstd();
}

fn print_cmc_listings(listings: &CMCListingResponse) {
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Price"]);
    for price in listings.data.iter() {
        table.add_row(row![price.symbol, format!("${:.5}", match price.quote.get("USD") { Some(p) => p.price, None => 0.0 })]);
    }
    table.printstd();
}

fn matches() -> clap::ArgMatches<'static> {
    let matches = clap_app!(coin =>
        (version: "1.0")
        (author: "Michael Paris <parisml@protonmail.com>")
        (about: "A CLI for interacting with Cryptocurrency Exchanges")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@subcommand account =>
            (about: "Show positions and their values")
            (version: "1.0")
            (@arg verbose: -v --verbose "Print test information verbosely")
        )
        (@subcommand cmc =>
            (about: "List current prices from CoinMarketCap")
            (version: "1.0")
        )
        (@subcommand save =>
            (about: "Save your current account information to airtable")
            (version: "1.0")
        )
        (@subcommand symbols =>
            (about: "List the symbols that can trade with a given currency")
            (version: "1.0")
            (@arg base: -b --base +takes_value "The base currency for the given trading symbols")
        )
        (@subcommand binance =>
            (about: "Print prices for assets on the binance exchange.")
            (version: "1.0")
        )
        (@subcommand config =>
            (about: "Print config information")
            (version: "1.0")
        )
        (@subcommand balance =>
            (about: "balance the portfolio")
            (version: "1.0")
            (@arg base: -b --base +takes_value "Rebalances the portfolio using this currency as the base.")
            (@arg size: -s --size +takes_value "Specifies how many currencies should be included in the index. Defaults to 10.")
            (@arg lookback: -l --lookback +takes_value "Specifies how many periods to lookback when calculating the moving average. Defaults to 20.")
            (@arg factor: -f --factor +takes_value "Specifies the smoothing factor for the moving average calculation. Defaults to 0.3.")
            (@arg mock: -m --mock "Preview the balance event but do not execute any trades.")
        )
        (@subcommand sell =>
            (about: "Sell one asset for another asset.")
            (version: "1.0")
            (@arg amount: +takes_value +required "Specify how much of the asset you would like to sell. Use the string all to sell as much of the asset as possible.")
            (@arg asset: +takes_value +required "Specify the symbol of the asset that you would like to sell.")
            (@arg into: -f --for +takes_value "Specify which asset you would like to sell into.")
        )
        (@subcommand buy =>
            (about: "Buy one asset --with another asset.")
            (version: "1.0")
            (@arg amount: -a --amount +takes_value "Specify how much should be spent in terms of the base currency. Use the string all to buy as much ETH as possible.")
            (@arg asset: +takes_value +required "Specify the symbol of the asset that you would like to buy.")
            (@arg with: -w --with +takes_value "Specify which currency you would like to use to buy the asset. Defaults to BTC.")
        )
    ).get_matches();
    matches
}
#[derive(Deserialize, Serialize, Debug)]
struct Account {
    balances: Vec<AccountBalance>,
    total_usd: f64,
    total_btc: f64,
}
impl Account {
    fn usd(&self) -> f64 {
        (self.total_usd * USD_FORMAT_MULTIPLIER).round()/USD_FORMAT_MULTIPLIER
    }
    fn btc(&self) -> f64 {
        (self.total_btc * BTC_FORMAT_MULTIPLIER).round()/BTC_FORMAT_MULTIPLIER
    }
}
#[derive(Deserialize, Serialize, Debug)]
struct AccountBalance {
    symbol: String,
    quantity: f64,
    value_usd: f64,
    value_btc: f64,
    change_7d: f64,
    change_24h: f64
}
impl AccountBalance {
    fn usd(&self) -> f64 {
        (self.value_usd * USD_FORMAT_MULTIPLIER).round()/USD_FORMAT_MULTIPLIER
    }
    fn btc(&self) -> f64 {
        (self.value_btc * BTC_FORMAT_MULTIPLIER).round()/BTC_FORMAT_MULTIPLIER
    }
}
fn make_account(balances: &Vec<Balance>, prices: CMCListingResponse) -> Result<Account> {
    let price_map = cmc_listings_as_map(prices);
    let price_btc = match price_map.get("BTC") {
        Some(price) => match price.quote.get("USD") {
            Some(quote) => quote.price,
            None => bail!("Could not find BTC price")
        },
        None => bail!("Could not find BTC price")
    };
    let mut total_usd = 0.0;
    let mut total_btc = 0.0;
    let mut acct_balances: Vec<AccountBalance> = Vec::new();
    balances.iter().for_each(
        |item| {
            let increase_7d = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.percent_change_7d,
                    None => 0.0
                },
                None => 0.0
            };
            let increase_24h = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.percent_change_24h,
                    None => 0.0
                },
                None => 0.0
            };
            let total_value = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.price * item.total(),
                    None => 0.0
                },
                None => 0.0
            };
            total_usd = total_usd + total_value;
            let total_value_btc = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.price * item.total() / price_btc,
                    None => 0.0
                },
                None => 0.0
            };
            total_btc = total_btc + total_value_btc;
            if item.total() > 0.0 && total_value > 1.0 {
                // total_value = (total_value * USD_FORMAT_MULTIPLIER).round()/USD_FORMAT_MULTIPLIER;
                // total_value_btc = (total_value_btc * BTC_FORMAT_MULTIPLIER).round()/BTC_FORMAT_MULTIPLIER;
                acct_balances.push(
                    AccountBalance {
                        symbol: item.symbol.to_string(),
                        quantity: item.total(),
                        value_usd: total_value,
                        value_btc: total_value_btc,
                        change_7d: increase_7d,
                        change_24h: increase_24h
                    }
                );
            }
        }
    );
    acct_balances.sort_unstable_by(
        |a, b| if a.value_usd > b.value_usd { std::cmp::Ordering::Less }
        else if a.value_usd == b.value_usd { std::cmp::Ordering::Equal }
        else { std::cmp::Ordering::Greater });
    Ok(Account {
        balances: acct_balances,
        total_btc: total_btc,
        total_usd: total_usd
    })
}

fn print_account(account: &Account) {
    let mut table = Table::new();
    table.add_row(row!("Symbol", "Quantity", "Value (USD)", "Value (BTC)", "Change (7d)", "Change (14d)"));
    account.balances.iter().for_each(
        |item| {
            let increase_7d = if item.change_7d > 0.0 {
                item.change_7d.to_string().green()
            } else {
                item.change_7d.to_string().red()
            };
            let increase_24h = if item.change_24h > 0.0 {
                item.change_24h.to_string().green()
            } else {
                item.change_24h.to_string().red()
            };
            if item.value_btc > 0.0 && item.value_usd > 1.0 {
                table.add_row(row![
                    item.symbol,
                    item.quantity.to_string().yellow(),
                    item.usd().to_string().blue(),
                    item.btc().to_string().cyan(),
                    increase_7d,
                    increase_24h
                ]);
            }
        }
    );
    table.add_row(row!["","",account.usd(), account.btc(),"",""]);
    table.printstd();
}

fn cmc_listings_as_map<'a>(listing: CMCListingResponse) -> HashMap<String, CMCListing> {
    let mut h_map = HashMap::new();
    for l in listing.data {
        h_map.insert(l.symbol.to_owned(), l);
    }
    h_map
}
