use crate::errors::*;

/**
 * Responsible for general market information
 */
pub trait Market {

  fn buy(&self) -> Result<()>;

  fn sell(&self) -> Result<()>;

  fn list_assets(&self) -> Result<()>;

  fn cost_basis(&self) -> Result<()>;

  fn capital_gains(&self) -> Result<()>;
}
