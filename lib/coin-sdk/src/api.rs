use crate::model::*;
use crate::errors::*;
use crate::sdk::SDK;

pub trait Coin {
    fn new(config: CoinConfig) -> Result<Box<Self>>;
}

impl Coin for SDK {
    fn new(config: CoinConfig) -> Result<Box<Self>> {
        Ok(Box::new(SDK::new(config)?))
    }
}