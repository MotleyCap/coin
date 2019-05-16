#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

pub mod model;
pub mod api;
pub mod errors;
pub mod sdk;
pub mod account;
pub mod binance;
pub mod coinbase;
pub mod coinbasepro;
mod client_factory;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
