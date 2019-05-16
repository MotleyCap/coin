use crate::errors::*;
use crate::model::{AccountConfig};

/**
 * The account is the bridge abstraction.
 * https://sourcemaking.com/design_patterns/bridge/python/1
 */
pub trait Account {

  fn buy(&self) -> Result<()>;

  fn sell(&self) -> Result<()>;

  fn list_assets(&self) -> Result<()>;

  fn cost_basis(&self) -> Result<()>;

  fn capital_gains(&self) -> Result<()>;
}

// impl Account for CoinAccount {
//   fn new(config: AccountConfig) -> Self {
//     CoinAccount {
//       config
//     }
//   }
// }