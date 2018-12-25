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
        let base_currency = _matches.value_of("base").unwrap_or("BTC");
        let index_size = _matches.value_of("size").unwrap_or("10");
        let index_size_i: u64 = index_size.parse().unwrap();
        let lookback = _matches.value_of("lookback").unwrap_or("20");
        let lookback_i: u64 = lookback.parse().unwrap();
        let factor = _matches.value_of("factor").unwrap_or("0.3");
        let factor_i: f64 = factor.parse().unwrap();
        let balanced_portfolio = balance_by_market_cap(&cmc, index_size_i, lookback_i, factor_i);
        print_portfolio(&balanced_portfolio);
        let order_ids = binance.enter_market(base_currency.to_owned(), &balanced_portfolio);
        let order_ids_str = order_ids.iter().map(|o| o.to_string()).collect::<Vec<String>>();
        println!("Successfully rebalanced positions with order_ids: [{}]", order_ids_str.join(", ").blue());
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

fn balance_by_market_cap(cmc: &CMCClient, index_size: u64, lookback: u64, smoothing_factor: f64) -> HashMap<String, f64> {
    let prices = cmc.latest_listings(100);
    let mut market_caps = HashMap::new();
    let known_assets: Vec<& str> = vec!["ada","ae","aion","ant","bat","bnb","btc","btg","btm","cennz","ctxc","cvc","dai","dash","dcr","dgb","doge","drgn","elf","eng","eos","etc","eth","ethos","fun","gas","gno","gnt","gusd","icn","icx","kcs","knc","loom","lrc","lsk","ltc","maid","mana","mtl","nas","neo","omg","pax","pay","pivx","poly","powr","ppt","qash","rep","rhoc","salt","snt","srn","ven","veri","vtc","waves","wtc","xem","xlm","xmr","xrp","xvg","zec","zil","zrx"];
    let mut seen_assets = 0;
    let mut table = Table::new();
    table.add_row(row!["Symbol", "Count", "Historical Market Caps"]);
    for price in prices.data {
        if seen_assets >= index_size {
            break;
        }
        let l_symbol = &price.symbol.to_lowercase()[..];
        if !known_assets.contains(&l_symbol) {
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
        market_caps.insert(price.symbol.to_owned(), historical_market_caps);
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

fn print_portfolio(allotments: &HashMap<String, f64>) {
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
            let increase_7d = match price_map.get(&item.asset) {
                Some(price) => match price.quote.get("USD") {
                    Some(quote) => if quote.percent_change_7d > 0.0 { quote.percent_change_7d.to_string().green() } else { quote.percent_change_7d.to_string().red() } ,
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