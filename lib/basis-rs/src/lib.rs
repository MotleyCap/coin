#[macro_use]
extern crate error_chain;

pub mod errors;
pub mod basis;

#[cfg(test)]
mod tests {
    use crate::basis::{Basis,BasisImpl};
    use crate::errors::*;

    #[test]
    fn test_cost_basis_and_gains() -> Result<()> {
        let mut basis: BasisImpl = Basis::new();
        basis.add_cost(100.0, 10.0);
        basis.realize_gain(100.0, 15.0);
        assert_eq!(basis.calc_cost_basis()?, 1000.0);
        assert_eq!(basis.calc_capital_gain()?, 500.0);
        basis.add_cost(30.0, 10.0);
        assert_eq!(basis.calc_cost_basis()?, 1000.0);
        assert_eq!(basis.calc_capital_gain()?, 500.0);
        basis.realize_gain(30.0, 15.0);
        assert_eq!(basis.calc_cost_basis()?, 1300.0);
        assert_eq!(basis.calc_capital_gain()?, 650.0);
        Ok(())
    }

    fn test_transfer_balance() -> Result<()> {
        let mut account1: BasisImpl = Basis::new();
        let mut account2: BasisImpl = Basis::new();
        account1.add_cost(100.0, 10.0);
        account1.add_cost(100.0, 15.0);
        assert_eq!(account1.calc_cost_basis()?, 0.0);
        let transfer = account1.transfer_basis(50.0)?;
        for t in transfer {
            account2.add_cost(t.quantity, t.value);
        }
        account2.realize_gain(50.0, 12.0);
        assert_eq!(account2.calc_cost_basis()?, 500.0);
        assert_eq!(account2.calc_capital_gain()?, 100.0);
        account1.realize_gain(100.0, 15.0);
        assert_eq!(account1.calc_cost_basis()?, 1250.0);
        assert_eq!(account1.calc_capital_gain()?, 250.0);
        Ok(())
    }
}
