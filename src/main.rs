use clap::clap_app;

mod clients;
mod portfolio;
use crate::clients::binance::{BinanceClient};
use crate::clients::client::{ExchangeOps,Balance};
use crate::clients::cmc::{CMCClient, CMCListingResponse, CMCListing};
use crate::portfolio::{Portfolio};
use std::env::{var};
use std::collections::HashMap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use colored::*;
use prettytable::{Table, Row, Cell, row, cell};

const BINANCE_KEY_NAME: &str = "BINANCE_KEY";
const BINANCE_SECRET_NAME: &str = "BINANCE_SECRET";
const CMC_KEY_NAME: &str = "CMC_KEY";

const INDEX_SMOOTHING_FACTOR: f64 = 0.3;
const INDEX_MOVING_AVERAGE_PERIOD: u64 = 30;
const INDEX_SIZE: u8 = 30;

fn main() {
    let matches = matches();
    let key = match var(BINANCE_KEY_NAME) {
        Ok(k) => k,
        Err(_) => panic!("Could not find binance key."),
    };
    let secret = match var(BINANCE_SECRET_NAME) {
        Ok(k) => k,
        Err(_) => panic!("Could not find binance secret."),
    };
    let cmc_key = match var(CMC_KEY_NAME) {
        Ok(k)=> k,
        Err(_) => panic!("Could not find CoinMarketCap key.")
    };
    let binance = BinanceClient::new(&key[..], &secret[..]);
    let cmc = CMCClient::new(cmc_key);
    if let Some(_matches) = matches.subcommand_matches("list") {
        let balances = binance.list();
        let prices = cmc.latest_listings(100);
        print_balances(Box::leak(balances), prices);
    } else if let Some(_matches) = matches.subcommand_matches("prices") {
        let prices = cmc.latest_listings(100);
        prices.data.iter().for_each(
            |item| println!("{}: ${}", item.symbol, match item.quote.get("USD") { Some(p) => p.price, None => 0.0 })
        )
    } else if let Some(_matches) = matches.subcommand_matches("balance") {
        let balanced_portfolio = balance_by_market_cap(&cmc, INDEX_MOVING_AVERAGE_PERIOD);
        print_portfolio(balanced_portfolio);
    } else {
        println!("Unknown command");
    }
}

fn balance_by_market_cap(cmc: &CMCClient, n: u64) -> HashMap<String, f64> {
    let prices = cmc.latest_listings(100);
    let mut market_caps = HashMap::new();
    let known_assets: Vec<& str> = vec!["ada","ae","aion","ant","bat","bch","bnb","bsv","btc","btg","btm","cennz","ctxc","cvc","dai","dash","dcr","dgb","doge","drgn","elf","eng","eos","etc","eth","ethos","fun","gas","gno","gnt","gusd","icn","icx","kcs","knc","loom","lrc","lsk","ltc","maid","mana","mtl","nas","neo","omg","pax","pay","pivx","poly","powr","ppt","qash","rep","rhoc","salt","snt","srn","trx","tusd","usdc","usdt","ven","veri","vtc","waves","wtc","xem","xlm","xmr","xrp","xvg","zec","zil","zrx"];
    let mut seen_assets = 0;
    for price in prices.data {
        if seen_assets >= INDEX_SIZE {
            break;
        }
        let l_symbol = &price.symbol.to_lowercase()[..];
        if !known_assets.contains(&l_symbol) {
            continue;
        }
        let historical_quotes = cmc.historic_quotes(&price.symbol, INDEX_MOVING_AVERAGE_PERIOD, "daily");
        let historical_market_caps = historical_quotes.result.iter().map(
            |h_quote| {
                let price = h_quote.1;
                price
            }
        ).collect::<Vec<f64>>();
        market_caps.insert(price.symbol, historical_market_caps);
        seen_assets = seen_assets + 1;
    }
    let portfolio = Portfolio::new(market_caps, INDEX_SMOOTHING_FACTOR);
    let allotments = portfolio.balance_by_market_cap();
    allotments
}

fn print_portfolio(allotments: HashMap<String, f64>) {
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Percentage"]);
    for (symbol, percentage) in allotments {
        table.add_row(row![symbol, format!("{:.2}", percentage * 100.0)]);
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
        (@subcommand prices =>
            (about: "list recent prices")
            (version: "1.0")
        )
        (@subcommand balance =>
            (about: "balance the portfolio")
            (version: "1.0")
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
            let increase_7d = match price_map.get(&item.asset) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => if quote.percent_change_24h > 0.0 { quote.percent_change_7d.to_string().green() } else { quote.percent_change_7d.to_string().red() } ,
                    None => "0".to_string().white()
                },
                None => "0".to_string().white()
            };
            let increase_24h = match price_map.get(&item.asset) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => if quote.percent_change_24h > 0.0 { quote.percent_change_24h.to_string().green() } else { quote.percent_change_24h.to_string().red() } ,
                    None => "0".to_string().white()
                },
                None => "0".to_string().white()
            };
            let total_value = match price_map.get(&item.asset) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => quote.price * item.total(),
                    None => 0.0
                },
                None => 0.0
            };
            if item.total() > 0.0 && total_value > 1.0 {
                table.add_row(row![
                    item.asset,
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