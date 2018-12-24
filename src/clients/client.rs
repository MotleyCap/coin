pub trait ExchangeOps {
  fn list(&self) -> Box<Vec<Balance>>;
}

pub struct Balance {
  pub asset: String,
  pub free: f64,
  pub locked: f64,
}

impl Balance {
  pub fn total(&self) -> f64 {
    self.free + self.locked
  }
}
