use crate::errors::*;
use crate::model::{AccountConfig, Asset};

/**
 * The account is the bridge abstraction.
 * https://sourcemaking.com/design_patterns/bridge/python/1
 */
pub trait Account {

  fn name(&self) -> &str;

  fn buy(&self) -> Result<()>;

  fn sell(&self) -> Result<()>;

  fn list_assets(&self) -> Result<Vec<Asset>>;

  fn cost_basis(&self) -> Result<()>;

  fn capital_gains(&self) -> Result<()>;
}
