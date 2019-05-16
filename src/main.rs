// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

use dirs::home_dir;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use clap::clap_app;

mod airtable;
mod binance;
mod cmc;
mod coinbase;
mod coinbasepro;
mod market_cap_balancer;
mod model;
mod persist;

use crate::airtable::{AirtableClient, AirtableConfig};
use crate::binance::BinanceClient;
use crate::cmc::{CMCClient, CMCListing, CMCListingResponse};
use crate::coinbase::CoinbaseClient;
use crate::coinbasepro::CoinbaseProClient;
use crate::market_cap_balancer::MarketCapBalancer;
use crate::model::{Account, ExchangeOps, Portfolio, PortfolioBalance, Price};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;

use colored::*;
use prettytable::{cell, row, Table};

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
pub mod errors {
    error_chain! {
        links {
            Coinbase(::coinbase::errors::Error, coinbase::errors::ErrorKind);
        }
        foreign_links {
            Reqwest(::reqwest::Error);
            ParseError(::std::num::ParseFloatError);
        }
    }
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
pub use crate::errors::*;

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
        use error_chain::ChainedError;
        use std::io::Write; // trait which holds `display_chain`
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
    let raw_config: Result<Config> = get_config();
    let config: Config = match raw_config {
        Err(e) => bail!(Error::with_chain(e, "Error loading config.")),
        Ok(c) => c,
    };
    // let key = config.binance.key.to_owned();
    // let secret = config.binance.secret.to_owned();
    let cmc_key = config.cmc.key.to_owned();
    let blacklisted_symbols: HashSet<_> = match &config.blacklist {
        Some(l) => {
            let as_set: HashSet<String> = l.iter().map(|s| s.to_string()).collect();
            as_set
        }
        None => HashSet::new(),
    };
    let default_airtable_config = AirtableConfig {
        key: "".to_string(),
        app: "".to_string(),
        table: "".to_string(),
        column_map: None,
    };
    let airtable_config = match &config.airtable {
        Some(atc) => &atc,
        None => &default_airtable_config,
    };
    let airtable = if (&airtable_config.key).len() > 0 && (&airtable_config.app).len() > 0 {
        Some(AirtableClient::new(&airtable_config))
    } else {
        None
    };
    let account_clients = get_account_clients(&config.account)?;
    if account_clients.len() == 0 {
        bail!(
            "You must provide at least one pair of binance credentials. {}",
            config.account.len()
        );
    }
    let binance_read_client = account_clients.first().unwrap();
    let cmc = CMCClient::new(cmc_key);
    if let Some(_matches) = matches.subcommand_matches("account") {
        let prices = cmc.latest_listings(100);
        let balances = if let Some(accounts_to_list) = _matches.values_of("name") {
            let mut valid_clients: Vec<Box<ExchangeOps>> = Vec::new();
            let mut acct_set: HashSet<&str> = HashSet::new();
            for acct_to_list in accounts_to_list {
                acct_set.insert(acct_to_list);
            }
            for client in account_clients {
                if acct_set.contains((*client).name()) {
                    valid_clients.push(client);
                }
            }
            // let vec_of_accounts: Vec<&str> = accounts_to_list.collect();
            // let valid_clients: Vec<BinanceClient> = account_clients.iter().filter(|c| vec_of_accounts.contains(&c.name)).collect();
            get_all_accounts(valid_clients)
        } else {
            get_all_accounts(account_clients)
        };
        match make_portfolio(&balances, &prices) {
            Ok(acct) => {
                print_portfolio(&acct);
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else if let Some(_matches) = matches.subcommand_matches("save") {
        let prices = cmc.latest_listings(100);
        for account_client in &account_clients {
            match account_client.all_accounts() {
                Ok(balances) => match make_portfolio(&balances, &prices) {
                    Ok(account) => {
                        if let Some(a_t) = &airtable {
                            save_account(&a_t, &account, (*account_client).name());
                        } else {
                            println!("Could not find airtable credentials in ~/.coin.yaml");
                        }
                    }
                    Err(e) => bail!(e),
                },
                Err(e) => bail!(e),
            }
        }
        let all_balances = get_all_accounts(account_clients);
        match make_portfolio(&all_balances, &prices) {
            Ok(account) => {
                if let Some(a_t) = &airtable {
                    save_account(&a_t, &account, "ALL");
                } else {
                    println!("Could not find airtable credentials in ~/.coin.yaml");
                }
            }
            Err(e) => bail!(e),
        }
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("symbols") {
        let base_currency = _matches
            .value_of("base")
            .unwrap_or("BTC")
            .to_ascii_uppercase();
        let tradable_symbols = get_tradeable_symbols(
            &base_currency,
            &blacklisted_symbols,
            binance_read_client,
            &cmc,
        );
        println!("Tradable symbols: {:?}", tradable_symbols);
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("cmc") {
        let prices = cmc.latest_listings(100);
        print_cmc_listings(&prices);
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("config") {
        println!(
            "{}",
            toml::to_string_pretty(&config).unwrap_or("Could not find ~/.coin.env".to_owned())
        );
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("binance") {
        match binance_read_client.all_prices() {
            Ok(prices) => {
                print_prices(prices);
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else if let Some(_matches) = matches.subcommand_matches("balance") {
        let base_currency = _matches.value_of("base").unwrap_or("BTC").to_uppercase();
        if base_currency != "BTC" {
            bail!("BTC is currently the only supported base currency for balance operations.")
        }
        let index_size = _matches.value_of("size").unwrap_or("20");
        let index_size_i: u64 = index_size.parse().unwrap();
        let lookback = _matches.value_of("lookback").unwrap_or("20");
        let lookback_i: u64 = lookback.parse().unwrap();
        let factor = _matches.value_of("factor").unwrap_or("0.3");
        let factor_i: f64 = factor.parse().unwrap();
        let is_mock = _matches.is_present("mock");
        // Find all pairs that trade with the base pai
        let tradable_symbols = get_tradeable_symbols(
            &base_currency,
            &blacklisted_symbols,
            binance_read_client,
            &cmc,
        )?;
        // First exit the market to the base currency.
        for trading_client in &account_clients {
            if !trading_client.can_trade() {
                continue;
            }
            if !is_mock {
                if let Ok(orders) = trading_client.exit_market(base_currency.to_owned()) {
                    let order_ids_str = orders
                        .iter()
                        .map(|o| o.id.to_string())
                        .collect::<Vec<String>>();
                    println!(
                        "Successfully exited old positions with order_ids: [{}]",
                        order_ids_str.join(", ").blue()
                    );
                }
            }
            let cmc_prices = cmc.latest_listings(100);
            let balanced_portfolio = balance_by_market_cap(
                &cmc,
                &cmc_prices.data,
                index_size_i,
                lookback_i,
                factor_i,
                &tradable_symbols,
            );
            print_asset_allocations(&balanced_portfolio);
            // Calculating total value
            if !is_mock {
                match trading_client.enter_market(base_currency.to_owned(), &balanced_portfolio) {
                    Ok(vec_of_orders) => {
                        let order_ids_str = vec_of_orders
                            .iter()
                            .map(|o| o.id.to_string())
                            .collect::<Vec<String>>();
                        if let Some(a_t) = &airtable {
                            if let Ok(balances) = trading_client.all_accounts() {
                                // let account = make_portfolio(&balances, cmc_prices);
                                if let Ok(account) = make_portfolio(&balances, &cmc_prices) {
                                    save_account(&a_t, &account, trading_client.name())
                                }
                            } else {
                                println!("Error saving to airtable");
                            }
                        }
                        println!(
                            "Successfully entered new positions with order_ids: [{}]",
                            order_ids_str.join(", ").blue()
                        );
                    }
                    Err(e) => println!(
                        "Failed to enter market for account {}\n{:?}",
                        trading_client.name().red(),
                        e
                    ),
                }
            }
        }
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("buy") {
        let amount_to_buy: f64 = match _matches.value_of("amount") {
            Some(a) => {
                if let Ok(_a) = a.parse::<f64>() {
                    _a
                } else {
                    bail!("Invalid amount {}", a)
                }
            }
            None => bail!("You must provide an amount to buy."),
        };
        let asset_to_buy = match _matches.value_of("asset") {
            Some(at) => at.to_uppercase(),
            None => bail!("You must provide the symbol of the asset you want to buy."),
        };
        let asset_to_buy_with = _matches.value_of("with").unwrap_or("BTC").to_uppercase();
        let account_to_buy_with = _matches.value_of("name").unwrap();
        if let Some(trading_client) = &account_clients
            .iter()
            .find(|i| (*i).name() == account_to_buy_with)
        {
            match trading_client.market_buy(asset_to_buy, asset_to_buy_with, amount_to_buy) {
                Ok(order) => {
                    println!("Successfully bought {:?}", order);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    } else if let Some(_matches) = matches.subcommand_matches("coinbase") {
        if let Some(coinbase_client) = account_clients.iter().find(|i| (*i).name() == "coinbase") {
            coinbase_client.all_accounts();
        }
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("watch") {
        let args: Vec<String> = env::args().collect();
        println!("{:?}", _matches);
        if let Some(_) = _matches.subcommand_matches("start") {
            println!("Trying to start daemon process");
            let mut child = Command::new(&args[0])
                .args(&["watch", "daemon"])
                .spawn()
                .expect("Child process failed to start.");
            println!("child pid: {}", child.id());
        // let result = child.wait();
        // println!("Child exited with result {:?}", result);
        } else if let Some(_) = _matches.subcommand_matches("kill") {
            println!("Would try to kill process.");
        } else if let Some(_) = _matches.subcommand_matches("daemon") {
            println!("In daemon");
            loop {
                thread::sleep(Duration::new(5, 0));
                println!("This is an incredibly simple daemon!");
            }
        } else {
            println!("Processing args {:?}", args);
        }
        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("cost") {
        let client = CoinbaseClient::new(
            "yeWEa818KVm8MUhw".to_owned(),
            "16R1zffQjMuziEqjZiPedEwzhuhlPSJm".to_owned(),
            "cb".to_owned(),
            true,
        );
        let buys = client.list_all_buys()?;
        let sum = buys.iter().fold(0f64, |acc, buy| acc + buy.total.amount);
        println!("Cost basis {}", sum);
        Ok(())
    } else {
        bail!("Unknown command")
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    pub blacklist: Option<Vec<String>>,
    pub account: Vec<AccountConfig>,
    pub cmc: CMCConfig,
    pub airtable: Option<AirtableConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AccountConfig {
    pub name: Option<String>,
    pub key: String,
    pub secret: String,
    pub passphrase: Option<String>,
    pub readonly: Option<bool>,
    pub service: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CMCConfig {
    pub key: String,
}
fn get_config() -> Result<Config> {
    let coin_file = if let Some(p) = home_dir() {
        match fs::read_to_string(p.join(".coin.toml")) {
            Ok(contents) => Some(contents),
            _ => bail!("Error reading ~/.coin.toml"),
        }
    } else {
        bail!("Could not find ~/.coin.toml")
    };
    match &coin_file {
        Some(contents) => {
            let conf: Config = match toml::from_str(&contents) {
                Ok(conts) => conts,
                Err(e) => bail!(Error::with_chain(e, "Error parsing .coin.toml")),
            };
            Ok(conf)
        }
        None => bail!("Could not find ~/.coin.toml"),
    }
}

fn get_account_clients(configs: &Vec<AccountConfig>) -> Result<Vec<Box<ExchangeOps>>> {
    let mut vec_of_clients: Vec<Box<ExchangeOps>> = Vec::new();
    for config in configs {
        let is_read_only = config.readonly.unwrap_or(false);
        let name: String = config.name.as_ref().map_or(&config.key, |i| i).to_owned();
        match &config.service[..] {
            "binance" => vec_of_clients.push(Box::new(BinanceClient::new(
                config.key.to_owned(),
                config.secret.to_owned(),
                name,
                is_read_only,
            ))),
            "coinbase" => vec_of_clients.push(Box::new(CoinbaseClient::new(
                config.key.to_owned(),
                config.secret.to_owned(),
                name,
                is_read_only,
            ))),
            "coinbasepro" => {
                if let Some(passphrase) = &config.passphrase {
                    vec_of_clients.push(Box::new(CoinbaseProClient::new(
                        config.key.to_owned(),
                        config.secret.to_owned(),
                        passphrase.to_owned(),
                        name,
                        is_read_only,
                    )));
                } else {
                    bail!("Coinbase Pro accounts must have a passphrase.")
                }
            }
            _ => continue,
        }
    }
    Ok(vec_of_clients)
}

fn get_all_accounts(clients: Vec<Box<ExchangeOps>>) -> Vec<Account> {
    // let all_balances: Vec<Account> = Vec::new();
    let mut all_balances: HashMap<String, Account> = HashMap::new();
    for client in clients {
        let client_account = match client.all_accounts() {
            Ok(balances) => Some(balances),
            Err(_) => None,
        };
        if let Some(account) = client_account {
            for balance in account {
                if let Some(existing_balance) = all_balances.get(&balance.asset[..]) {
                    let new_balance = Account {
                        available: existing_balance.available + balance.available,
                        asset: existing_balance.asset.to_owned(),
                        locked: existing_balance.locked + balance.locked,
                    };
                    all_balances.insert(balance.asset.to_owned(), new_balance);
                } else {
                    all_balances.insert(balance.asset.to_owned(), balance);
                }
            }
        } else {
            println!("Could not fetch balances for account: {}", client.name());
        }
    }
    all_balances
        .iter()
        .map(|(_, val)| val.clone())
        .collect::<Vec<Account>>()
}

fn get_tradeable_symbols(
    base_currency: &str,
    blacklist: &HashSet<String>,
    account: &Box<ExchangeOps>,
    cmc: &CMCClient,
) -> Result<HashSet<String>> {
    match (*account).all_prices() {
        Ok(prices) => {
            let mut tradable_symbols: HashSet<_> = prices
                .iter()
                .filter(|item| {
                    item.symbol.ends_with(&base_currency) || item.symbol.starts_with(&base_currency)
                })
                .map(|item| match item.symbol.ends_with(&base_currency) {
                    true => item.symbol[0..item.symbol.len() - base_currency.len()].to_owned(),
                    false => item.symbol[base_currency.len()..item.symbol.len()].to_owned(),
                })
                .collect();
            let coins_with_data = cmc.supported_assets();
            println!(
                "Fetched {} supported assets from api.coinmetrics.com",
                coins_with_data.len()
            );
            if !tradable_symbols.contains(base_currency) {
                tradable_symbols.insert(base_currency.to_string());
            }
            let tradable_symbols: HashSet<String> = tradable_symbols
                .difference(blacklist)
                .map(|s| s.to_string())
                .collect();
            let tradable_symbols: HashSet<String> = tradable_symbols
                .intersection(&coins_with_data)
                .map(|s| s.to_string())
                .collect();
            Ok(tradable_symbols)
        }
        Err(e) => Err(e),
    }
}

#[derive(Deserialize, Serialize)]
struct AccountRecord {
    total_usd: f64,
    total_btc: f64,
    timestamp: String,
    details: String,
    name: String,
}
fn save_account(airtable: &AirtableClient, account: &Portfolio, name: &str) {
    let now = Utc::now();
    let account_str = serde_json::to_string_pretty(&account.balances).unwrap();
    let value = AccountRecord {
        total_usd: account.total_usd,
        total_btc: account.total_btc,
        name: name.to_owned(),
        timestamp: now.to_string(),
        details: account_str,
    };
    let value_json = serde_json::to_value(value).unwrap();
    match airtable.create_record(value_json) {
        Ok(_) => println!("Successfully saved portfolio to airtable."),
        Err(e) => println!(
            "Error saving portfolio to airtable: ${}",
            e.status().unwrap()
        ),
    };
}

fn balance_by_market_cap(
    cmc: &CMCClient,
    prices: &Vec<CMCListing>,
    index_size: u64,
    lookback: u64,
    smoothing_factor: f64,
    tradable_assets: &HashSet<String>,
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
        if historical_quotes.result.len() == 0 {
            println!(
                "Could not find market cap information for {}",
                &price.symbol
            );
            continue;
        }
        // Slow rate to 5 reqs a second
        // let throttle_length = time::Duration::from_millis(200);
        // thread::sleep(throttle_length);
        let historical_market_caps = historical_quotes
            .result
            .iter()
            .map(|h_quote| {
                let price = h_quote.1;
                price
            })
            .collect::<Vec<f64>>();
        let symbol = &price.symbol[..];
        let values_as_string = historical_market_caps
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let len = historical_market_caps.len();
        market_caps.insert((&price.symbol).to_owned(), historical_market_caps);
        table.add_row(row![symbol, len, values_as_string,]);
        seen_assets = seen_assets + 1;
    }
    let balancer = MarketCapBalancer::new(market_caps, smoothing_factor);
    let allotments = balancer.balance_by_market_cap();
    table.printstd();
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

fn print_asset_allocations(allotments: &HashMap<String, f64>) {
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
        table.add_row(row![
            price.symbol,
            format!(
                "${:.5}",
                match price.quote.get("USD") {
                    Some(p) => p.price,
                    None => 0.0,
                }
            )
        ]);
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
            (@arg name: -n --name +takes_value +multiple "Specify accounts to list details about.")
        )
        (@subcommand cmc =>
            (about: "List current prices from CoinMarketCap")
            (version: "1.0")
        )
        (@subcommand coinbase =>
            (about: "List current prices from Coinbase")
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
            (about: "Balance your portfolio holdings according to ~/coin.toml")
            (version: "1.0")
            (@arg base: -b --base +takes_value "Rebalances the portfolio using this currency as the base.")
            (@arg size: -s --size +takes_value "Specifies how many currencies should be included in the index. Defaults to 10.")
            (@arg lookback: -l --lookback +takes_value "Specifies how many periods to lookback when calculating the moving average. Defaults to 20.")
            (@arg factor: -f --factor +takes_value "Specifies the smoothing factor for the moving average calculation. Defaults to 0.3.")
            (@arg mock: -m --mock "Preview the balance event but do not execute any trades.")
        )
        (@subcommand cost =>
            (about: "Compute cost basis")
            (version: "1.0")
        )
    ).get_matches();
    matches
}
fn make_portfolio(accounts: &Vec<Account>, prices: &CMCListingResponse) -> Result<Portfolio> {
    let price_map = cmc_listings_as_map(prices);
    let price_btc = match price_map.get("BTC") {
        Some(price) => match price.quote.get("USD") {
            Some(quote) => quote.price,
            None => bail!("Could not find BTC price"),
        },
        None => bail!("Could not find BTC price"),
    };
    let mut total_usd = 0.0;
    let mut total_btc = 0.0;
    let mut acct_balances: Vec<PortfolioBalance> = Vec::new();
    let flat_accounts = summarize_accounts(accounts);
    flat_accounts.iter().for_each(|item| {
        let increase_7d = match price_map.get(&item.asset) {
            Some(price) => match price.quote.get("USD") {
                Some(quote) => quote.percent_change_7d,
                None => 0.0,
            },
            None => 0.0,
        };
        let increase_24h = match price_map.get(&item.asset) {
            Some(price) => match price.quote.get("USD") {
                Some(quote) => quote.percent_change_24h,
                None => 0.0,
            },
            None => 0.0,
        };
        let total_value = match price_map.get(&item.asset) {
            Some(price) => match price.quote.get("USD") {
                Some(quote) => quote.price * item.total(),
                None => 0.0,
            },
            None => 0.0,
        };
        total_usd = total_usd + total_value;
        let total_value_btc = match price_map.get(&item.asset) {
            Some(price) => match price.quote.get("USD") {
                Some(quote) => quote.price * item.total() / price_btc,
                None => 0.0,
            },
            None => 0.0,
        };
        total_btc = total_btc + total_value_btc;
        if item.total() > 0.0 && total_value > 1.0 {
            // total_value = (total_value * USD_FORMAT_MULTIPLIER).round()/USD_FORMAT_MULTIPLIER;
            // total_value_btc = (total_value_btc * BTC_FORMAT_MULTIPLIER).round()/BTC_FORMAT_MULTIPLIER;
            acct_balances.push(PortfolioBalance {
                symbol: item.asset.to_string(),
                quantity: item.total(),
                value_usd: total_value,
                value_btc: total_value_btc,
                change_7d: increase_7d,
                change_24h: increase_24h,
            });
        }
    });
    acct_balances.sort_unstable_by(|a, b| {
        if a.value_usd > b.value_usd {
            std::cmp::Ordering::Less
        } else if a.value_usd == b.value_usd {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    });
    Ok(Portfolio {
        balances: acct_balances,
        total_btc: total_btc,
        total_usd: total_usd,
    })
}

fn summarize_accounts(accounts: &Vec<Account>) -> Vec<Account> {
    let mut asset_map: HashMap<String, Account> = HashMap::new();
    for account in accounts {
      if let Some(val) = asset_map.get(&account.asset) {
        let updated_balance = Account {
            asset: account.asset.to_string(),
            available: val.available + account.available,
            locked: val.locked + account.locked
        };
        asset_map.insert(account.asset.to_string(), updated_balance);
      } else {
        asset_map.insert(account.asset.to_string(), Account {
            asset: account.asset.to_string(),
            available: account.available,
            locked: account.locked
        });
      }
    }
    let balances: Vec<Account> = asset_map.into_iter().map(|(_,v)| v).collect();
    balances
}

fn print_portfolio(account: &Portfolio) {
    let mut table = Table::new();
    table.add_row(row!(
        "Symbol",
        "Quantity",
        "Value (USD)",
        "Value (BTC)",
        "Change (7d)",
        "Change (14d)"
    ));
    account.balances.iter().for_each(|item| {
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
    });
    table.add_row(row!["", "", account.usd(), account.btc(), "", ""]);
    table.printstd();
}

fn cmc_listings_as_map<'a>(listing: &'a CMCListingResponse) -> HashMap<String, &'a CMCListing> {
    let mut h_map = HashMap::new();
    for l in &listing.data {
        h_map.insert(l.symbol.to_owned(), l);
    }
    h_map
}
