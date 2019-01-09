use std::fs;
use std::path::{Path};
use std::env::{var,home_dir};
use std::collections::{HashMap,HashSet};

use clap::clap_app;
use chrono::prelude::*;

mod clients;
mod portfolio;
use crate::clients::binance::{BinanceClient};
use crate::clients::client::{ExchangeOps,Balance,Price};
use crate::clients::cmc::{CMCClient, CMCListingResponse, CMCListing};
use crate::clients::airtable::{AirtableClient};
use crate::portfolio::{Portfolio};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;

use colored::*;
use prettytable::{Table, row, cell};

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

#[derive(Deserialize, Serialize, Debug)]
struct AirtableConfig {
    pub key: String,
    pub space: String,
}


fn main() {
    let matches = matches();
    let coin_file = if let Some(p) = home_dir() {
        match fs::read_to_string(p.join(".coin.toml")) {
            Ok(contents) => {
                Some(contents)
            },
            Err(e) => None
        }
    } else {
        None
    };
    let raw_config: Option<Config> = match &coin_file {
        Some(contents) => {
            let conf: Config = match toml::from_str(&contents) {
                Ok(conts) => conts,
                Err(e) => {
                    panic!(e);
                }
            };
            Some(conf)
        },
        None => None
    };
    let config: Config = match raw_config {
        None => panic!("Could not find ~/.coin.toml"),
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
    let airtable_config = config.airtable.unwrap_or(AirtableConfig { key: "".to_string(), space: "".to_string()});
    let airtable = if (&airtable_config.key).len() > 0 && (&airtable_config.space).len() > 0 {
        Some(AirtableClient::new(&airtable_config.key, &airtable_config.space))
    } else {
        None
    };
    let binance = BinanceClient::new(&key[..], &secret[..]);
    let cmc = CMCClient::new(cmc_key);
    if let Some(_matches) = matches.subcommand_matches("balances") {
        let balances = binance.list();
        let prices = cmc.latest_listings(100);
        print_balances(Box::leak(balances), prices);
    } else if let Some(_matches) = matches.subcommand_matches("symbols") {
        let binance_prices = binance.all_prices();
        let base_currency = _matches.value_of("base").unwrap_or("BTC").to_ascii_uppercase();
        let tradable_symbols: HashSet<_> = binance_prices
            .iter()
            .filter(
                |item| item.symbol.ends_with(&base_currency) || item.symbol.starts_with(&base_currency)
            ).map(
                |item| match item.symbol.ends_with(&base_currency) {
                    true => item.symbol[0..item.symbol.len()-base_currency.len()].to_owned(),
                    false => item.symbol[base_currency.len()..item.symbol.len()].to_owned()
                }
            ).collect();
        let tradable_symbols: HashSet<String> = tradable_symbols.difference(&blacklisted_symbols).map(|s| s.to_string()).collect();
        println!("Tradable symbols: {:?}", tradable_symbols);
    } else if let Some(_matches) = matches.subcommand_matches("cmc") {
        let prices = cmc.latest_listings(100);
        print_cmc_listings(&prices);
    } else if let Some(_matches) = matches.subcommand_matches("config") {
        println!("{:?}", coin_file);
    } else if let Some(_matches) = matches.subcommand_matches("prices") {
        let prices = binance.all_prices();
        print_prices(Box::leak(prices));
    } else if let Some(_matches) = matches.subcommand_matches("balance") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC").to_ascii_uppercase();
        let index_size = _matches.value_of("size").unwrap_or("10");
        let index_size_i: u64 = index_size.parse().unwrap();
        let lookback = _matches.value_of("lookback").unwrap_or("20");
        let lookback_i: u64 = lookback.parse().unwrap();
        let factor = _matches.value_of("factor").unwrap_or("0.3");
        let factor_i: f64 = factor.parse().unwrap();
        // Find all pairs that trade with the base pair
        let binance_prices = binance.all_prices();
        let mut tradable_symbols: HashSet<_> = binance_prices
            .iter()
            .filter(
                |item| item.symbol.ends_with(&base_currency)
            ).map(
                |item| item.symbol[0..item.symbol.len()-base_currency.len()].to_owned()
            ).collect();

        let base_currency_copy = base_currency[..].to_owned();
        if !tradable_symbols.contains(&base_currency_copy) {
            tradable_symbols.insert(base_currency_copy);
        }
        let tradable_symbols: HashSet<String> = tradable_symbols.difference(&blacklisted_symbols).map(|s| s.to_string()).collect();
        println!("Acceptable trading symbols: {:?}", tradable_symbols);
        // First exit the market to the base currency.
        let order_ids = binance.exit_market(base_currency.to_owned());
        let order_ids_str = order_ids.iter().map(|o| o.to_string()).collect::<Vec<String>>();
        println!("Successfully exited old positions with order_ids: [{}]", order_ids_str.join(", ").blue());
        let cmc_prices = cmc.latest_listings(100);
        let balanced_portfolio = balance_by_market_cap(&cmc, &cmc_prices.data, index_size_i, lookback_i, factor_i, tradable_symbols);
        print_portfolio(&balanced_portfolio);
        // Calculating total value
        let base_worth = binance.position(base_currency.to_owned());
        let base_price = cmc_prices.data.iter().find(|&p| p.symbol.to_uppercase() == base_currency.to_uppercase()).unwrap();
        let usd_worth = base_price.quote.get("USD").unwrap().price * base_worth;
        let order_ids = binance.enter_market(base_currency.to_owned(), &balanced_portfolio);
        let order_ids_str = order_ids.iter().map(|o| o.to_string()).collect::<Vec<String>>();
        if let Some(a_t) = airtable {
            save_portfolio(&a_t, balanced_portfolio, usd_worth, base_worth);
        }
        println!("Successfully entered new positions with order_ids: [{}]", order_ids_str.join(", ").blue());
    } else if let Some(_matches) = matches.subcommand_matches("exit") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC");
        if _matches.is_present("position") {
            let position_to_exit = _matches.value_of("position").unwrap();
            let order_id = binance.market_sell_all(position_to_exit.to_owned(), base_currency.to_owned());
            println!("Successfully exited {} position with order_id: {}", position_to_exit, order_id.to_string().green());
        } else {
            let order_ids = binance.exit_market(base_currency.to_owned());
            let order_ids_str = order_ids.iter().map(|o| o.to_string()).collect::<Vec<String>>();
            println!("Successfully exited all positions with order_ids: [{}]", order_ids_str.join(", ").blue());
        }
    } else if let Some(_matches) = matches.subcommand_matches("enter") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC");
        let amount = if _matches.is_present("amount") {
            let as_float: f64 = match _matches.value_of("amount").unwrap().parse() {
                Ok(o) => o,
                Err(e) => panic!("{} is not a valid amount.", _matches.value_of("amount").unwrap())
            };
            as_float
        } else {
            binance.position(base_currency.to_owned())
        };
        let position_to_enter = _matches.value_of("position").unwrap();
        let order_id = binance.market_buy(position_to_enter.to_owned(), base_currency.to_owned(), amount);
        println!("Successfully entered position with order_id: {}", order_id.to_string().green());
    } else {
        println!("Unknown command");
    }
}

fn save_portfolio(airtable: &AirtableClient, portfolio: HashMap<String, f64>, value_usd: f64, value_btc: f64) {
    let now = Utc::now();
    let position_str = serde_json::to_string(&portfolio).unwrap();
    let value = serde_json::json!({
        "Timestamp": now.to_string(),
        "Value (USD)": value_usd,
        "Value (BTC)": value_btc,
        "Positions": position_str
    });
    airtable.create_record("Portfolio History".to_owned(), &value);
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

fn print_prices(prices: &mut Vec<Price>) {
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
        (@subcommand list =>
            (about: "list owned assets")
            (version: "1.0")
            (@arg verbose: -v --verbose "Print test information verbosely")
        )
        (@subcommand cmc =>
            (about: "list recent prices from cmc in USD")
            (version: "1.0")
        )
        (@subcommand save =>
            (about: "test save portfolio")
            (version: "1.0")
        )
        (@subcommand symbols =>
            (about: "List the symbols that can trade with a given currency")
            (version: "1.0")
            (@arg base: -b --base +takes_value "The base currency for the given trading symbols")
        )
        (@subcommand prices =>
            (about: "print exchange prices")
            (version: "1.0")
        )
        (@subcommand config =>
            (about: "print config information")
            (version: "1.0")
        )
        (@subcommand balance =>
            (about: "balance the portfolio")
            (version: "1.0")
            (@arg base: -b --base +takes_value "Rebalances the portfolio using this currency as the base.")
            (@arg size: -s --size +takes_value "Specifies how many currencies should be included in the index. Defaults to 10.")
            (@arg lookback: -l --lookback +takes_value "Specifies how many periods to lookback when calculating the moving average. Defaults to 20.")
            (@arg factor: -f --factor +takes_value "Specifies the smoothing factor for the moving average calculation. Defaults to 0.3.")
        )
        (@subcommand exit =>
            (about: "exit positions by selling into a single base currency")
            (version: "1.0")
            (@arg position: -p --position +takes_value "Specify a single position to exit. If omitted, all positions are exited.")
            (@arg base: -b --base +takes_value "Exit all positions selling into a single base currency. Defaults to BTC.")
        )
        (@subcommand enter =>
            (about: "enter a position by buying a currency with a base currency")
            (version: "1.0")
            (@arg position: -p --position +takes_value +required "Specify a single position to enter. If omitted, all positions are exited.")
            (@arg amount: -a --amount +takes_value "Specify how much should be spent in terms of the base currency. If omitted, the entire base currency position will be used.")
            (@arg base: -b --base +takes_value "Exit all positions selling into a single base currency. Defaults to BTC.")
        )
    ).get_matches();
    matches
}

fn print_balances(balances: &mut Vec<Balance>, prices: CMCListingResponse) {
    let price_map = cmc_listings_as_map(prices);
    balances.sort_unstable_by(
        |a, b| if a.total() > b.total() { std::cmp::Ordering::Less }
        else if a.total() == b.total() { std::cmp::Ordering::Equal }
        else { std::cmp::Ordering::Greater });
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Shares", "Value (USD)", "% change (7d)", "% change (24 h)"]);
    balances.iter().for_each(
        |item| {
            let increase_7d = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => if quote.percent_change_7d > 0.0 { quote.percent_change_7d.to_string().green() } else { quote.percent_change_7d.to_string().red() } ,
                    None => "0".to_string().white()
                },
                None => "0".to_string().white()
            };
            let increase_24h = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => if quote.percent_change_24h > 0.0 { quote.percent_change_24h.to_string().green() } else { quote.percent_change_24h.to_string().red() } ,
                    None => "0".to_string().white()
                },
                None => "0".to_string().white()
            };
            let total_value = match price_map.get(&item.symbol) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.price * item.total(),
                    None => 0.0
                },
                None => 0.0
            };
            if item.total() > 0.0 && total_value > 1.0 {
                table.add_row(row![
                    item.symbol,
                    item.total().to_string().yellow(),
                    total_value.to_string().blue(),
                    increase_7d,
                    increase_24h
                ]);
            }
        }
    );
    table.printstd();
}

fn cmc_listings_as_map<'a>(listing: CMCListingResponse) -> HashMap<String, CMCListing> {
    let mut h_map = HashMap::new();
    for l in listing.data {
        h_map.insert(l.symbol.to_owned(), l);
    }
    h_map
}