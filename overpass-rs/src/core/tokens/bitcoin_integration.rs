use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    traits::{Currency, WithdrawReasons, ExistenceRequirement},
    pallet_prelude::Member,
    Parameter,
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

impl<T: BitcoinConfig> Bitcoin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn deposit(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::Balance,
    ) -> DispatchResult {
        match currency_id {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::deposit_creating(who, amount);
                Ok(())
            },
            _ => Err(DispatchError::Other("Unsupported currency")),
        }
    }

    pub fn withdraw(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::Balance,
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

    pub fn can_slash(currency_id: BitcoinNetwork, who: &T::AccountId, amount: <<T::NativeCurrency as Currency<T::AccountId>>::Balance>) -> bool {
        match currency_id {
            BitcoinNetwork::Bitcoin => T::NativeCurrency::can_slash(who, amount),
            _ => false,
        }
    }

    pub fn slash(
        currency_id: BitcoinNetwork,
        who: &T::AccountId,
        amount: <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::Balance,
    ) -> <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::Balance {
        match currency_id {
            BitcoinNetwork::Bitcoin => {
                let (_, slashed) = T::NativeCurrency::slash(who, amount);
                match T::NativeCurrency::withdraw(
                    who,
                    amount,
                    WithdrawReasons::all(),
                    ExistenceRequirement::AllowDeath,
                ) {
                    Ok(_) => amount,
                    Err(_) => <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::default(),
                }
            },
            _ => <<T::NativeCurrency as Currency<T::AccountId>>::Balance>::default(),
        }
    }
}

#[derive(Debug)]
pub struct PositiveImbalance<T: BitcoinConfig>(<<T::NativeCurrency as Currency<T::AccountId>>::Balance>);      

impl<T: BitcoinConfig> Default for PositiveImbalance<T> {
    fn default() -> Self {
        Self(<<T::NativeCurrency as Currency<T::AccountId>>::Balance>::default(), PhantomData)
    }
}

#[derive(Debug)]
pub struct NegativeImbalance<T: BitcoinConfig>(<<T::NativeCurrency as Currency<T::AccountId>>::Balance>, PhantomData<T>);

impl<T: BitcoinConfig> Default for NegativeImbalance<T> {
    fn default() -> Self {
        Self(<<T::NativeCurrency as Currency<T::AccountId>>::Balance>::default(), PhantomData)
    }
}

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
        ) -> Result<Self::PositiveImbalance, frame_support::traits::BalanceStatus> {
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
        let bitcoin = Bitcoin::<TestConfig>::new();
        let account_id = 1_u64;
        let amount = 100_u128;

        assert!(Bitcoin::<TestConfig>::deposit(BitcoinNetwork::Bitcoin, &account_id, amount).is_ok());
        assert!(Bitcoin::<TestConfig>::withdraw(BitcoinNetwork::Bitcoin, &account_id, amount).is_ok());
        assert!(Bitcoin::<TestConfig>::deposit(BitcoinNetwork::BitcoinTestnet, &account_id, amount).is_err());
    }
}