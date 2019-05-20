use crate::errors::*;
use crate::model::{AccountConfig, Asset, Amount};

/**
 * The account is the bridge abstraction.
 * https://sourcemaking.com/design_patterns/bridge/python/1
 */
pub trait Account {

  fn name(&self) -> &str;

  fn buy(&self) -> Result<()>;

  fn sell(&self) -> Result<()>;

  fn list_assets(&self) -> Result<Vec<Asset>>;

  fn total_costs(&self) -> Result<Amount>;

  fn total_gains(&self) -> Result<Amount>;

  fn cost_basis(&self) -> Result<()>;

  fn capital_gains(&self) -> Result<()>;
}
