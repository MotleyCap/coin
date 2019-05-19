use std::collections::{HashMap};
use binance::api::*;
use binance::account::{Account as AccountImpl};
use binance::market::{Market};
use binance::model::{Prices};
use crate::model::{AccountConfig, Asset};
use crate::errors::*;
use crate::account::Account;

pub struct BinanceAccount {
  config: AccountConfig,
  account: AccountImpl,
  market: Market,
  rounding: HashMap<String, i32>,
}

impl BinanceAccount {
  pub fn new(config: AccountConfig) -> Self {
    let account = Binance::new(Some(config.key.to_string()), Some(config.secret.to_string()));
    let market: Market = Binance::new(None, None);
    BinanceAccount {
      config,
      account,
      market,
      rounding: make_rounding_rules(),
    }
  }
}

impl Account for BinanceAccount {

  fn name(&self) -> &str {
    &self.config.name
  }

  fn buy(&self) -> Result<()> {
    Ok(())
  }

  fn sell(&self) -> Result<()> {
    Ok(())
  }

  fn list_assets(&self) -> Result<Vec<Asset>> {
    let balances = match self.account.get_account() {
      Ok(answer) => answer.balances,
      Err(e) => bail!("Error fetching balances: {}", e),
    };
    let coerced = balances
      .iter()
      .map(|bal| Asset {
        asset: bal.asset.to_owned(),
        available: bal.free.parse().unwrap(),
        locked: bal.locked.parse().unwrap()
      })
      .collect::<Vec<Asset>>();
    Ok(coerced)
  }

  fn cost_basis(&self) -> Result<()> {
    Ok(())
  }

  fn capital_gains(&self) -> Result<()> {
    Ok(())
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