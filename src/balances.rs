use num::traits::{CheckedAdd, CheckedSub, Zero};
use std::collections::BTreeMap;

pub enum BalanceError<AccountId> {
    InsufficientBalance,
    BalanceOverflow(AccountId),
}

pub trait Config: crate::system::Config {
    type Balance: Zero + CheckedSub + CheckedAdd + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
    balances: BTreeMap<T::AccountId, T::Balance>,
}

impl<T: Config> Pallet<T> {
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
        }
    }

    pub fn set_balance(&mut self, who: &T::AccountId, amount: T::Balance) {
        self.balances.insert(who.clone(), amount);
    }

    pub fn balance(&self, who: &T::AccountId) -> T::Balance {
        *self.balances.get(who).unwrap_or(&T::Balance::zero())
    }

    pub fn transfer(
        &mut self,
        caller: &T::AccountId,
        to: &T::AccountId,
        amount: &T::Balance,
    ) -> Result<(), BalanceError<T::AccountId>> {
        let mut caller_balance = self.balance(caller);
        let mut to_balance = self.balance(to);

        caller_balance = caller_balance
            .checked_sub(amount)
            .ok_or(BalanceError::InsufficientBalance)?;

        to_balance = to_balance
            .checked_add(amount)
            .ok_or(BalanceError::BalanceOverflow(to.clone()))?;

        self.set_balance(&caller, caller_balance);
        self.set_balance(&to, to_balance);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::balances::Config;

    struct BalanceConfig;
    impl Config for BalanceConfig {
        type Balance = u128;
    }

    impl crate::system::Config for BalanceConfig {
        type AccountId = String;
        type BlockNumber = u32;
        type Nonce = u32;
    }

    #[test]
    fn init_balances() {
        let mut balances = super::Pallet::<BalanceConfig>::new();

        assert_eq!(balances.balance(&"alice".to_string()), 0);
        balances.set_balance(&"alice".to_string(), 100);
        assert_eq!(balances.balance(&"alice".to_string()), 100);
        assert_eq!(balances.balance(&"bob".to_string()), 0);
    }

    #[test]
    fn transfer_balance() {
        let mut balances = super::Pallet::<BalanceConfig>::new();
        let alice = String::from("alice");
        let bob = String::from("bob");

        balances.set_balance(&alice, 100);
        assert_eq!(balances.balance(&alice), 100);

        let res = balances.transfer(&alice, &bob, &50);

        assert_eq!(res.is_ok(), true);
        assert_eq!(balances.balance(&alice), 50);
        assert_eq!(balances.balance(&bob), 50);
    }
}
