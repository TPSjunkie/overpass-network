// ./src/core/tokens/bitcoin_integration.rs
// This file is part of the Overpass Network.

// Copyright (C) 2020-2021 Overpass Network.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.   

// Breakdown of file:
// 1. BitcoinConfig - Trait for configuring Bitcoin-related types and constants.
// 2. BitcoinNetwork - Enum for representing different Bitcoin networks.
// 3. Bitcoin - Struct for managing Bitcoin operations.
// 4. PositiveImbalance - Struct for representing a positive imbalance.
// 5. NegativeImbalance - Struct for representing a negative imbalance.
// 6. tests - Unit tests for the Bitcoin struct.
// 7. MockNativeCurrency - Mock implementation of the NativeCurrency trait.

use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Member, traits::{Currency, ExistenceRequirement, Imbalance, WithdrawReasons}, Parameter
};
use sp_runtime::traits::AtLeast32BitUnsigned;
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;
use sp_runtime::DispatchError;

pub trait BitcoinConfig: 'static + Eq + Clone {
    type AccountId: Parameter + Member;
    type Balance: AtLeast32BitUnsigned + Parameter + Member + Default + Copy;
    type NativeCurrency: Currency<Self::AccountId>;
}

pub const BITCOIN_MAINNET: u8 = 0;
pub const BITCOIN_TESTNET: u8 = 1;
pub const BITCOIN_REGTEST: u8 = 2;
pub const BITCOIN_SIGNET: u8 = 3;
pub const BITCOIN_SIMNET: u8 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum BitcoinNetwork {
    Bitcoin,
    BitcoinTestnet,
    BitcoinRegtest,
    BitcoinSignet,
    BitcoinSimnet,
}

impl BitcoinNetwork {
    pub fn to_string(&self) -> String {
        match self {
            BitcoinNetwork::Bitcoin => "Bitcoin".to_string(),
            BitcoinNetwork::BitcoinTestnet => "Bitcoin Testnet".to_string(),
            BitcoinNetwork::BitcoinRegtest => "Bitcoin Regtest".to_string(),
            BitcoinNetwork::BitcoinSignet => "Bitcoin Signet".to_string(),
            BitcoinNetwork::BitcoinSimnet => "Bitcoin Simnet".to_string(),
        }
    }
}

impl Default for BitcoinNetwork {
    fn default() -> Self {
        BitcoinNetwork::Bitcoin
    }
}

pub struct Bitcoin<T: BitcoinConfig>(PhantomData<T>);

impl<T: BitcoinConfig> Default for Bitcoin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: BitcoinConfig> Bitcoin<T> where T::NativeCurrency: Currency<T::AccountId> {
    /// Create a new instance of the Bitcoin class. 
    /// 
    /// # Arguments
    ///
    /// * `currency_id` - The currency ID to use for the Bitcoin instance.
    ///
    /// # Returns
    ///
    /// A new instance of the Bitcoin class.  
    pub fn new(currency_id: BitcoinNetwork) -> Self {
        Self(PhantomData)
    }

    pub fn deposit(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
    ) -> DispatchResult {
        match currency_id {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::deposit_creating(who, amount);
                Ok(())
            },
            _ => Err(DispatchError::Other("Unsupported currency")),
        }
    }

    /// Withdraw the given amount from the given account.
    ///
    /// # Arguments
    ///
    /// * `currency_id` - The currency ID to use for the Bitcoin instance.
    /// * `who` - The account to withdraw from.
    /// * `amount` - The amount to withdraw.
    ///
    /// # Returns
    ///
    /// A Result containing an error if the withdrawal failed, or Ok(()) if the withdrawal succeeded. 
    pub fn withdraw(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
    ) -> Result<(), DispatchError> {
        match currency_id {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::withdraw(
                    who,
                    amount,
                    WithdrawReasons::all(),
                    ExistenceRequirement::KeepAlive,
                ).map(|_| ())
            },
            _ => Err(DispatchError::Other("Unsupported currency")),
        }
    }

    pub fn can_slash(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
    ) -> bool {
        match currency_id {
            BitcoinNetwork::Bitcoin => T::NativeCurrency::can_slash(who, amount),
            _ => false,
        }
    }


    /// Slash the given amount from the given account.
    ///
    /// # Arguments
    ///
    /// * `currency_id` - The currency ID to use for the Bitcoin instance.
    /// * `who` - The account to slash.
    /// * `amount` - The amount to slash.
    ///
    /// # Returns
    ///
    /// The amount that was actually slashed.   
    pub fn slash(      
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
    ) -> <T::NativeCurrency as Currency<T::AccountId>>::Balance {      
        match currency_id {
            BitcoinNetwork::Bitcoin => {
                let (imbalance, _) = T::NativeCurrency::slash(who, amount);
                imbalance.peek()
            },
            _ => <T::NativeCurrency as Currency<T::AccountId>>::Balance::default(),
        }
    }       
}

/// A struct representing a positive imbalance.
#[derive(Debug)]
pub struct PositiveImbalance<T: BitcoinConfig>(T::Balance, PhantomData<T>);      

impl<T: BitcoinConfig> Default for PositiveImbalance<T> {
    fn default() -> Self {
        Self(T::Balance::default(), PhantomData)
    }
}

impl<T: BitcoinConfig> PositiveImbalance<T> {
    pub fn new(amount: T::Balance) -> Self {
        Self(amount, PhantomData)
    }
}

/// A struct representing a negative imbalance.
#[derive(Debug)]
pub struct NegativeImbalance<T: BitcoinConfig>(T::Balance, PhantomData<T>);

impl<T: BitcoinConfig> NegativeImbalance<T> {
    pub fn new(amount: T::Balance) -> Self {
        Self(amount, PhantomData)
    }
}

/// Unit tests for the Bitcoin struct.
#[cfg(test)]
mod tests {
    use frame_support::traits::SignedImbalance;
    use super::*;

    #[derive(Clone, Eq, PartialEq)]
    pub struct TestConfig;

    impl BitcoinConfig for TestConfig {
        type AccountId = u64;
        type Balance = u128;
        type NativeCurrency = MockNativeCurrency;
    }

    /// A mock implementation of the NativeCurrency trait.
    /// 
    /// This struct is used for testing purposes only.
    /// It provides a default implementation for all methods, allowing the tests to run without any errors.
    /// 
    /// # Example
    /// 
    /// ```
    /// use crate::core::tokens::bitcoin_integration::{BitcoinConfig, MockNativeCurrency};
    /// use frame_support::traits::Currency;
    /// 
    /// #[derive(Clone, Eq, PartialEq)]
    /// pub struct TestConfig;
    /// 
    /// impl BitcoinConfig for TestConfig {
    ///     type AccountId = u64;
    ///     type Balance = u128;
    ///     type NativeCurrency = MockNativeCurrency;
    /// }
    /// 
    /// let mock_native_currency = MockNativeCurrency;
    /// let account_id = 1_u64;
    /// let amount = 100_u128;
    /// 
    /// assert_eq!(mock_native_currency.total_balance(&account_id), 0);
    /// assert!(mock_native_currency.can_slash(&account_id, amount));
    /// assert_eq!(mock_native_currency.total_issuance(), 0);
    /// assert_eq!(mock_native_currency.minimum_balance(), 0);
    /// assert_eq!(mock_native_currency.burn(amount), mock_native_currency.PositiveImbalance::new(amount));
    /// assert_eq!(mock_native_currency.issue(amount), mock_native_currency.NegativeImbalance::new(amount));
    /// assert_eq!(mock_native_currency.free_balance(&account_id), 0);
    /// assert!(mock_native_currency.ensure_can_withdraw(&account_id, amount, WithdrawReasons::all(), amount).is_ok());
    /// assert_eq!(mock_native_currency.transfer(&account_id, &account_id, amount, ExistenceRequirement::KeepAlive).is_ok(), ());
    /// assert_eq!(mock_native_currency.slash(&account_id, amount), (mock_native_currency.NegativeImbalance::new(amount), 0));
    /// assert_eq!(mock_native_currency.deposit_into_existing(&account_id, amount).is_ok(), ());
    /// assert_eq!(mock_native_currency.deposit_creating(&account_id, amount), mock_native_currency.PositiveImbalance::new(amount));
    /// assert_eq!(mock_native_currency.withdraw(&account_id, amount, WithdrawReasons::all(), ExistenceRequirement::KeepAlive).is_ok(), ());
    /// assert_eq!(mock_native_currency.make_free_balance_be(&account_id, amount).peek(), mock_native_currency.SignedImbalance::Positive(mock_native_currency.PositiveImbalance::new(amount)));
    /// ```
    /// 
    /// # Panics
    /// 
    /// This function panics if the `currency_id` is not supported.
    pub struct MockNativeCurrency;

    impl Currency<u64> for MockNativeCurrency {
        type Balance = u128;
        type PositiveImbalance = ();
        type NegativeImbalance = ();

        fn total_balance(_who: &u64) -> Self::Balance { 0 }
        fn can_slash(_who: &u64, _value: Self::Balance) -> bool { true }
        fn total_issuance() -> Self::Balance { 0 }
        fn minimum_balance() -> Self::Balance { 0 }
        fn burn(_amount: Self::Balance) -> Self::PositiveImbalance { () }
        fn issue(_amount: Self::Balance) -> Self::NegativeImbalance { () }
        fn free_balance(_who: &u64) -> Self::Balance { 0 }
        fn ensure_can_withdraw(
            _who: &u64,
            _amount: Self::Balance,
            _reasons: WithdrawReasons,
            _new_balance: Self::Balance,
        ) -> DispatchResult { Ok(()) }
        fn transfer(
            _source: &u64,
            _dest: &u64,
            _value: Self::Balance,
            _existence_requirement: ExistenceRequirement,
        ) -> DispatchResult { Ok(()) }
        fn slash(
            _who: &u64,
            _value: Self::Balance,
        ) -> (Self::NegativeImbalance, Self::Balance) { ((), 0) }
        fn deposit_into_existing(
            _who: &u64,
            _value: Self::Balance,
        ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
            Ok(())
        }
        fn deposit_creating(_who: &u64, _value: Self::Balance) -> Self::PositiveImbalance {
            ()
        }
        fn withdraw(
            _who: &u64,
            _value: Self::Balance,
            _reasons: WithdrawReasons,
            _liveness: ExistenceRequirement,
        ) -> Result<Self::NegativeImbalance, DispatchError> { Ok(()) }
        fn make_free_balance_be(
            _who: &u64,
            _balance: Self::Balance,
        ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
            SignedImbalance::Positive(())
        }
    }

    #[test]
    fn test_bitcoin_network() {
        assert_eq!(BitcoinNetwork::default(), BitcoinNetwork::Bitcoin);
        let network = BitcoinNetwork::BitcoinTestnet;
        assert_ne!(network, BitcoinNetwork::Bitcoin);
    }

    #[test]
    fn test_bitcoin_operations() {
        let bitcoin = Bitcoin::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account_id = 1_u64;
        let amount = 100_u128;

        assert!(Bitcoin::<TestConfig>::deposit(BitcoinNetwork::Bitcoin, &account_id, amount).is_ok());
        assert!(Bitcoin::<TestConfig>::withdraw(BitcoinNetwork::Bitcoin, &account_id, amount).is_ok());
        assert!(Bitcoin::<TestConfig>::deposit(BitcoinNetwork::BitcoinTestnet, &account_id, amount).is_err());
    }
    #[test]
    #[should_panic]
    fn test_bitcoin_panic() {
        let bitcoin = Bitcoin::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account_id = 1_u64;
        let amount = 100_u128;

        Bitcoin::<TestConfig>::deposit(BitcoinNetwork::BitcoinTestnet, &account_id, amount);
    }
    #[test]
    fn test_bitcoin_slash() {
        let bitcoin = Bitcoin::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account_id = 1_u64;
        let amount = 100_u128;

        assert_eq!(Bitcoin::<TestConfig>::slash(BitcoinNetwork::Bitcoin, &account_id, amount), amount);
    }
}