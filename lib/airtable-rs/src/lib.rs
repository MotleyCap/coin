#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

pub mod client;
pub mod errors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
