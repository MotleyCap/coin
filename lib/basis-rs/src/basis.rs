use std::collections::VecDeque;
use crate::errors::*;

pub struct Amount {
  pub quantity: f64,
  pub value: f64,
}
pub struct BasisImpl {
  costs: VecDeque<Amount>,
  gains: VecDeque<Amount>,
}

pub trait Basis {
  fn new() -> Self;

  /**
   * Add a single cost basis entry.
   */
  fn add_cost(&mut self, quantity: f64, value: f64) -> &Self;

  /**
   * Removes cost basis entries starting at the oldest to transfer to another account.
   */
  fn transfer_basis(&mut self, quantity: f64) -> Result<Vec<Amount>>;

  /**
   * Registers a taxable event.
   */
  fn realize_gain(&mut self, quantity: f64, value: f64) -> &Self;

  /**
   * Returns the value of the cost basis for any realized returns.
   * Unrealized returns are not included in the cost basis.
   */
  fn calc_cost_basis(&mut self) -> Result<f64>;

  /**
   * Returns the value of the total capital gains for the realized returns.
   */
  fn calc_capital_gain(&mut self) -> Result<f64>;
}

impl Basis for BasisImpl {
  fn new() -> Self {
    BasisImpl {
      costs: VecDeque::new(),
      gains: VecDeque::new(),
    }
  }

  fn add_cost(&mut self, quantity: f64, value: f64) -> &Self {
    self.costs.push_back(Amount { quantity, value });
    self
  }

  fn transfer_basis(&mut self, quantity: f64) -> Result<Vec<Amount>> {
    let mut remaining_quantity = quantity;
    let mut transferring = Vec::new();
    while (remaining_quantity > 0.0) {
      if let Some(oldest_quantity) = self.costs.pop_front() {
        if oldest_quantity.quantity > remaining_quantity {
          self.costs.push_front(Amount { quantity: oldest_quantity.quantity - remaining_quantity, value: oldest_quantity.value });
          transferring.push(Amount { quantity: remaining_quantity, value: oldest_quantity.value });
          remaining_quantity = 0.0;
        } else {
          remaining_quantity -= oldest_quantity.quantity;
          transferring.push(Amount { quantity: oldest_quantity.quantity, value: oldest_quantity.value });
        }
      } else {
        return Err(Error::from_kind(ErrorKind::BasisError("There is not enough cost basis to support transfer.".to_string())));
      }
    }
    Ok(transferring)
  }

  fn realize_gain(&mut self, quantity: f64, value: f64) -> &Self {
    self.gains.push_back(Amount { quantity, value });
    self
  }

  fn calc_cost_basis(&mut self) -> Result<f64> {
    let mut cost_basis = 0.0;
    let mut cost_index = 0;
    let mut cost_index_consumed = 0.0;
    for amt in &self.gains {
      let mut remaining_gain_quantity = amt.quantity;
      while remaining_gain_quantity > 0.0 {
        if let Some(oldest_cost) = self.costs.get(cost_index) {
          let remaining_cost_quantity = oldest_cost.quantity - cost_index_consumed;
          if remaining_cost_quantity > remaining_gain_quantity {
            cost_basis += remaining_gain_quantity * oldest_cost.value;
            cost_index_consumed += remaining_gain_quantity;
            remaining_gain_quantity = 0.0;
          } else {
            cost_basis += remaining_cost_quantity * oldest_cost.value;
            remaining_gain_quantity -= remaining_cost_quantity;
            cost_index += 1;
            cost_index_consumed = 0.0;
          }
        } else {
          return Err(Error::from_kind(ErrorKind::BasisError("There is not enough cost basis to support gains.".to_string())));
        }
      }
    }
    Ok(cost_basis)
  }

  fn calc_capital_gain(&mut self) -> Result<f64> {
    let cost_basis = self.calc_cost_basis()?;
    let total_gains = self.gains.iter().fold(0f64, |acc, gain| acc + gain.quantity * gain.value);
    Ok(total_gains - cost_basis)
  }
}