use std::collections::HashMap;
use std::f64::consts::{E};

/**
 * The Portfolio is responsible for calculating how much should
 * be allotted to each currency in the index.
 */
pub struct Portfolio {
  // A map of symbol -> vec of market cap values for that symbol.
  // There is one number per day for as many days as should be included
  // in the exponential moving average calculation.
  market_caps: HashMap<String, Vec<f64>>,

  // The smoothing factor. 0 < alpha < 1 defaults to 0.3.
  // As you decrease alpha, you get a smoother line.
  alpha: f64,
}

impl Portfolio {

  pub fn new(caps: HashMap<String, Vec<f64>>, alpha: f64) -> Self {
    Portfolio {
      market_caps: caps,
      alpha: alpha
    }
  }

  /**
   * Calculate the percentage holdings for each symbol based on a exponentially
   * smoothed market cap weighting.
   */
  pub fn balance_by_market_cap(&self) -> HashMap<String, f64> {
    let smoothed_caps = self.smooth_market_caps();
    let mut total_sum = 0.0;
    for (_, cap) in &smoothed_caps {
      total_sum = total_sum + cap;
    }
    let mut percs = HashMap::new();
    for (symbol, cap) in &smoothed_caps {
      let percentage = cap / total_sum;
      percs.insert(symbol.to_owned(), percentage);
    }
    percs
  }

  /**
   * Calculate the smoothed market cap at some time t.
   * M*(t) = SUM_i_to_n( M(T-i)e^-(alpha*i) ) / SUM_i_to_n(e^-(alpha*i))
   */
  fn smooth_market_caps(&self) -> HashMap<String, f64> {
    let mut smoothed_caps = HashMap::new();
    for (symbol, caps) in &self.market_caps {
      let market_cap = self.smooth_market_cap(&caps);
      smoothed_caps.insert(symbol.to_owned(), market_cap);
    }
    smoothed_caps
  }

  /**
   * Calculate the smoothed market cap value for a single currency.
   */
  fn smooth_market_cap(&self, market_caps: &Vec<f64>) -> f64 {
    let total_cap_periods = market_caps.len();
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for i in 0..total_cap_periods {
      // If 10 periods this loops from 9 to 0.
      let mc_index = total_cap_periods - 1 - i;
      // M(T-i)
      let market_cap_at_t = market_caps[mc_index];
      // e ^ -(alpha * i)
      let exponential = E.powf(-1.0 * self.alpha * i as f64);
      numerator = numerator + (market_cap_at_t * exponential);
      denominator = denominator + exponential;
    }
    numerator / denominator
  }
}